//! Key input with precise timestamps.
//!
//! On macOS an `NSEvent` local monitor sees every key event of this app before
//! winit does, with the hardware event timestamp. No extra permission is
//! needed: it only sees events that are sent to this app anyway.
//!
//! The input knows nothing about notes. It emits raw [`KeyEvent`]s for the
//! physical key codes in its *play set* (the note and control keys, given by
//! the caller); what a key means lives in the app and the core crate.

use std::cell::RefCell;
use std::rc::Rc;

/// One key down or key up of a play key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// macOS virtual key code (physical position, ANSI numbering).
    pub code: u16,
    /// True for key down, false for key up.
    pub down: bool,
    /// Host time in ns (`mach_absolute_time` clock, the clock of the audio engine).
    pub host_ns: u64,
}

/// A set of macOS virtual key codes (0..128).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeySet(u128);

impl KeySet {
    /// The empty set.
    pub const EMPTY: KeySet = KeySet(0);

    /// A set of the given codes; codes ≥ 128 are ignored.
    pub fn from_codes(codes: &[u16]) -> KeySet {
        let mut s = KeySet::EMPTY;
        for c in codes {
            s.insert(*c);
        }
        s
    }

    /// True if `code` is in the set.
    pub fn contains(self, code: u16) -> bool {
        code < 128 && self.0 & (1u128 << code) != 0
    }

    /// Adds `code` (ignored when ≥ 128).
    pub fn insert(&mut self, code: u16) {
        if code < 128 {
            self.0 |= 1u128 << code;
        }
    }

    /// Removes `code`.
    pub fn remove(&mut self, code: u16) {
        if code < 128 {
            self.0 &= !(1u128 << code);
        }
    }

    /// True if no code is in the set.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The codes in ascending order.
    pub fn iter(self) -> impl Iterator<Item = u16> {
        (0u16..128).filter(move |c| self.contains(*c))
    }
}

/// What the monitor does with one raw key event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// A play-key event: emit it and swallow it (no beep, no UI shortcut).
    Emit(KeyEvent),
    /// A play key that must not reach the UI (key repeat, duplicate down).
    Swallow,
    /// Not ours: let the UI handle it (Space, Escape, text fields, shortcuts).
    Pass,
}

/// One raw key event from the system or from egui.
#[derive(Clone, Copy, Debug)]
pub struct RawKey {
    /// Virtual key code.
    pub code: u16,
    /// Key down (true) or key up (false).
    pub down: bool,
    /// The system marked a key down as auto-repeat.
    pub repeat: bool,
    /// Cmd, Ctrl or Option is held.
    pub command_modifier: bool,
    /// Event time in host ns.
    pub host_ns: u64,
}

/// The input rules, without any platform code:
/// only play keys, no repeats, no Cmd/Ctrl/Option, only while a play surface
/// is active, every key up for a held key, no double downs.
#[derive(Clone, Debug, Default)]
pub struct KeyFilter {
    /// Codes that are notes or controls.
    pub play: KeySet,
    /// A play surface has focus (false in text fields and on list screens).
    pub active: bool,
    held: KeySet,
}

impl KeyFilter {
    /// A filter for `play`, inactive until [`KeyFilter::active`] is set.
    pub fn new(play: KeySet) -> KeyFilter {
        KeyFilter {
            play,
            active: false,
            held: KeySet::EMPTY,
        }
    }

    /// Keys that are down now.
    pub fn held(&self) -> KeySet {
        self.held
    }

    /// Applies the rules to one raw event.
    pub fn on_raw(&mut self, k: RawKey) -> Verdict {
        if !self.play.contains(k.code) {
            return Verdict::Pass;
        }
        let ev = KeyEvent {
            code: k.code,
            down: k.down,
            host_ns: k.host_ns,
        };
        if !k.down {
            // A held key always gets its key up, even after focus moved or with a modifier.
            if self.held.contains(k.code) {
                self.held.remove(k.code);
                return Verdict::Emit(ev);
            }
            return if self.active {
                Verdict::Swallow
            } else {
                Verdict::Pass
            };
        }
        if !self.active || k.command_modifier {
            return Verdict::Pass;
        }
        if k.repeat || self.held.contains(k.code) {
            return Verdict::Swallow;
        }
        self.held.insert(k.code);
        Verdict::Emit(ev)
    }

    /// Key ups for every held key (focus loss, screen change); clears the held set.
    pub fn release_all(&mut self, host_ns: u64) -> Vec<KeyEvent> {
        let out = self
            .held
            .iter()
            .map(|code| KeyEvent {
                code,
                down: false,
                host_ns,
            })
            .collect();
        self.held = KeySet::EMPTY;
        out
    }
}

/// Called on the main thread for every emitted key event, before the UI frame
/// sees it, so sound starts with minimum delay.
pub type KeyHook = Box<dyn Fn(KeyEvent) + 'static>;

struct Shared {
    filter: KeyFilter,
    queue: Vec<KeyEvent>,
    hook: Option<KeyHook>,
}

impl Shared {
    fn handle(&mut self, raw: RawKey) -> Verdict {
        let v = self.filter.on_raw(raw);
        if let Verdict::Emit(ev) = v {
            if let Some(h) = &self.hook {
                h(ev);
            }
            self.queue.push(ev);
        }
        v
    }
}

/// The app's key input: the native monitor on macOS, egui events elsewhere and in tests.
pub struct KeyInput {
    shared: Rc<RefCell<Shared>>,
    native: bool,
    #[cfg(target_os = "macos")]
    _monitor: Option<macos::Monitor>,
}

impl KeyInput {
    /// Input filter and egui fallback without installing a native event monitor.
    pub fn offline(play: KeySet) -> Self {
        Self {
            shared: Rc::new(RefCell::new(Shared {
                filter: KeyFilter::new(play),
                queue: Vec::new(),
                hook: None,
            })),
            native: false,
            #[cfg(target_os = "macos")]
            _monitor: None,
        }
    }

    /// Starts listening for the codes in `play`. `LEARNAPP_NO_NATIVE_INPUT=1`
    /// turns the native monitor off (UI tests feed egui key events instead).
    pub fn new(play: KeySet) -> KeyInput {
        let shared = Rc::new(RefCell::new(Shared {
            filter: KeyFilter::new(play),
            queue: Vec::new(),
            hook: None,
        }));
        #[cfg(target_os = "macos")]
        {
            let monitor = if std::env::var_os("LEARNAPP_NO_NATIVE_INPUT").is_some() {
                None
            } else {
                macos::Monitor::install(shared.clone())
            };
            KeyInput {
                native: monitor.is_some(),
                shared,
                _monitor: monitor,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            KeyInput {
                shared,
                native: false,
            }
        }
    }

    /// True if events come from the native monitor (with hardware timestamps).
    pub fn is_native(&self) -> bool {
        self.native
    }

    /// Sets the hook for immediate sound.
    pub fn set_hook(&self, hook: KeyHook) {
        self.shared.borrow_mut().hook = Some(hook);
    }

    /// Replaces the play set (layout change).
    pub fn set_play_keys(&self, play: KeySet) {
        self.shared.borrow_mut().filter.play = play;
    }

    /// Turns note capture on (a play surface has focus) or off (lists, text fields).
    pub fn set_active(&self, active: bool) {
        self.shared.borrow_mut().filter.active = active;
    }

    /// Keys that are down now.
    pub fn held(&self) -> KeySet {
        self.shared.borrow().filter.held()
    }

    /// All key events since the last call, oldest first.
    pub fn drain(&self) -> Vec<KeyEvent> {
        std::mem::take(&mut self.shared.borrow_mut().queue)
    }

    /// Emits key ups for every held key (window lost focus). Returns them too.
    pub fn release_all(&self) -> Vec<KeyEvent> {
        let mut s = self.shared.borrow_mut();
        let ups = s.filter.release_all(host_now_ns());
        for ev in &ups {
            if let Some(h) = &s.hook {
                h(*ev);
            }
        }
        s.queue.extend(ups.iter().copied());
        ups
    }

    /// Injects a timestamped physical event through the native input filter and hook.
    pub fn inject(&self, event: KeyEvent) {
        self.shared.borrow_mut().handle(RawKey {
            code: event.code,
            down: event.down,
            repeat: false,
            command_modifier: false,
            host_ns: event.host_ns,
        });
    }

    /// Fallback without the native monitor (other platforms, UI tests): feeds
    /// egui key events, stamped with the current host time.
    pub fn feed_egui(&self, ctx: &egui::Context) {
        if self.native {
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        let mut s = self.shared.borrow_mut();
        for e in events {
            if let egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                modifiers,
                ..
            } = e
                && let Some(code) = mac_code_for_egui_key(physical_key.unwrap_or(key))
            {
                s.handle(RawKey {
                    code,
                    down: pressed,
                    repeat,
                    command_modifier: modifiers.command || modifiers.ctrl || modifiers.alt,
                    host_ns: host_now_ns(),
                });
            }
        }
    }
}

/// macOS virtual key code of an egui key (physical US position).
pub fn mac_code_for_egui_key(k: egui::Key) -> Option<u16> {
    use egui::Key::*;
    Some(match k {
        A => 0x00,
        S => 0x01,
        D => 0x02,
        F => 0x03,
        H => 0x04,
        G => 0x05,
        Z => 0x06,
        X => 0x07,
        C => 0x08,
        V => 0x09,
        B => 0x0B,
        Q => 0x0C,
        W => 0x0D,
        E => 0x0E,
        R => 0x0F,
        Y => 0x10,
        T => 0x11,
        O => 0x1F,
        U => 0x20,
        I => 0x22,
        P => 0x23,
        L => 0x25,
        J => 0x26,
        Quote => 0x27,
        K => 0x28,
        Semicolon => 0x29,
        N => 0x2D,
        M => 0x2E,
        _ => return None,
    })
}

/// Current host time in ns (same clock as NSEvent timestamps and cpal).
pub fn host_now_ns() -> u64 {
    #[cfg(target_os = "macos")]
    {
        macos::mach_now_ns()
    }
    #[cfg(not(target_os = "macos"))]
    {
        use std::time::Instant;
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        START.get_or_init(Instant::now).elapsed().as_nanos() as u64
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{RawKey, Shared, Verdict};
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType};
    use std::cell::RefCell;
    use std::ptr::NonNull;
    use std::rc::Rc;

    pub struct Monitor {
        token: Retained<AnyObject>,
        _block: RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent>,
    }

    impl Drop for Monitor {
        fn drop(&mut self) {
            // SAFETY: the token came from addLocalMonitorForEventsMatchingMask.
            unsafe { NSEvent::removeMonitor(&self.token) };
        }
    }

    impl Monitor {
        pub fn install(shared: Rc<RefCell<Shared>>) -> Option<Monitor> {
            let block = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
                // SAFETY: AppKit passes a valid NSEvent for the duration of the call.
                let ev = unsafe { event.as_ref() };
                let ty = ev.r#type();
                let down = ty == NSEventType::KeyDown;
                if !(down || ty == NSEventType::KeyUp) {
                    return event.as_ptr();
                }
                // Re-entrant call (a hook that pumps events): leave it to the UI.
                let Ok(mut s) = shared.try_borrow_mut() else {
                    return event.as_ptr();
                };
                let blocked = NSEventModifierFlags::Command
                    | NSEventModifierFlags::Control
                    | NSEventModifierFlags::Option;
                let raw = RawKey {
                    code: ev.keyCode(),
                    down,
                    // isARepeat is only defined for key events (checked above).
                    repeat: down && ev.isARepeat(),
                    command_modifier: ev.modifierFlags().intersects(blocked),
                    // NSEvent timestamps are seconds of system uptime = mach_absolute_time.
                    host_ns: (ev.timestamp() * 1e9) as u64,
                };
                match s.handle(raw) {
                    Verdict::Pass => event.as_ptr(),
                    Verdict::Emit(_) | Verdict::Swallow => std::ptr::null_mut(),
                }
            });
            let mask = NSEventMask::KeyDown | NSEventMask::KeyUp;
            // SAFETY: called on the main thread after NSApplication exists; the
            // block lives as long as the monitor (stored in Monitor).
            let token =
                unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block) }?;
            Some(Monitor {
                token,
                _block: block,
            })
        }
    }

    #[repr(C)]
    struct MachTimebaseInfo {
        numer: u32,
        denom: u32,
    }

    unsafe extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
    }

    pub fn mach_now_ns() -> u64 {
        static TB: std::sync::OnceLock<(u64, u64)> = std::sync::OnceLock::new();
        let (n, d) = *TB.get_or_init(|| {
            let mut info = MachTimebaseInfo { numer: 0, denom: 0 };
            // SAFETY: plain out-parameter call.
            unsafe { mach_timebase_info(&mut info) };
            (info.numer.max(1) as u64, info.denom.max(1) as u64)
        });
        // SAFETY: no arguments, always safe.
        let t = unsafe { mach_absolute_time() };
        (t as u128 * n as u128 / d as u128) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: u16 = 0x00;
    const SPACE: u16 = 0x31;

    fn raw(code: u16, down: bool) -> RawKey {
        RawKey {
            code,
            down,
            repeat: false,
            command_modifier: false,
            host_ns: 1_000,
        }
    }

    fn active() -> KeyFilter {
        let mut f = KeyFilter::new(KeySet::from_codes(&[A, 0x01]));
        f.active = true;
        f
    }

    #[test]
    fn keyset_basics() {
        let mut s = KeySet::from_codes(&[0, 5, 127, 200]);
        assert!(s.contains(0) && s.contains(5) && s.contains(127) && !s.contains(200));
        s.remove(5);
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![0, 127]);
        assert!(KeySet::EMPTY.is_empty());
    }

    #[test]
    fn play_key_down_up() {
        let mut f = active();
        assert_eq!(
            f.on_raw(raw(A, true)),
            Verdict::Emit(KeyEvent {
                code: A,
                down: true,
                host_ns: 1_000
            })
        );
        assert!(f.held().contains(A));
        assert!(matches!(
            f.on_raw(raw(A, false)),
            Verdict::Emit(KeyEvent { down: false, .. })
        ));
        assert!(f.held().is_empty());
    }

    #[test]
    fn other_keys_pass() {
        let mut f = active();
        assert_eq!(f.on_raw(raw(SPACE, true)), Verdict::Pass);
    }

    #[test]
    fn repeats_and_double_downs_are_swallowed() {
        let mut f = active();
        f.on_raw(raw(A, true));
        assert_eq!(
            f.on_raw(RawKey {
                repeat: true,
                ..raw(A, true)
            }),
            Verdict::Swallow
        );
        assert_eq!(f.on_raw(raw(A, true)), Verdict::Swallow);
    }

    #[test]
    fn modifiers_are_not_notes_but_release_held_keys() {
        let mut f = active();
        assert_eq!(
            f.on_raw(RawKey {
                command_modifier: true,
                ..raw(A, true)
            }),
            Verdict::Pass
        );
        f.on_raw(raw(A, true));
        assert!(matches!(
            f.on_raw(RawKey {
                command_modifier: true,
                ..raw(A, false)
            }),
            Verdict::Emit(_)
        ));
    }

    #[test]
    fn inactive_passes_downs_but_still_releases() {
        let mut f = active();
        f.on_raw(raw(A, true));
        f.active = false;
        assert_eq!(f.on_raw(raw(0x01, true)), Verdict::Pass);
        assert!(matches!(f.on_raw(raw(A, false)), Verdict::Emit(_)));
        assert_eq!(f.on_raw(raw(A, false)), Verdict::Pass);
    }

    #[test]
    fn release_all_on_focus_loss() {
        let mut f = active();
        f.on_raw(raw(A, true));
        f.on_raw(raw(0x01, true));
        let ups = f.release_all(9);
        assert_eq!(ups.len(), 2);
        assert!(ups.iter().all(|e| !e.down && e.host_ns == 9));
        assert!(f.held().is_empty());
    }

    #[test]
    fn egui_keys_map_to_physical_codes() {
        assert_eq!(mac_code_for_egui_key(egui::Key::A), Some(0x00));
        assert_eq!(mac_code_for_egui_key(egui::Key::Semicolon), Some(0x29));
        assert_eq!(mac_code_for_egui_key(egui::Key::Quote), Some(0x27));
        assert_eq!(mac_code_for_egui_key(egui::Key::Space), None);
    }

    #[test]
    fn host_clock_moves_forward() {
        let a = host_now_ns();
        let b = host_now_ns();
        assert!(b >= a && a > 0);
    }
}
