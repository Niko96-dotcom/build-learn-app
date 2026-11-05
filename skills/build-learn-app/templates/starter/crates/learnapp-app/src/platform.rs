//! Small macOS integrations: open a folder in Finder, keep the process out of
//! App Nap while it plays, and the background mode for verification.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static BACKGROUND: AtomicBool = AtomicBool::new(false);
static KEY_EVENTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Key events AppKit delivered to this app since launch (background mode
/// only; counted by a pass-through monitor for the background report).
pub fn key_events_seen() -> u64 {
    KEY_EVENTS.load(Ordering::Relaxed)
}

/// True in background mode (`--background` / `LEARNAPP_NO_ACTIVATE=1`).
pub fn background() -> bool {
    BACKGROUND.load(Ordering::Relaxed)
}

/// Background mode for driving the app while someone works on the same Mac.
///
/// Call once on the main thread before the event loop starts. From then on
/// the app never activates itself (`activateIgnoringOtherApps:` and
/// `activate` do nothing) and its windows are ordered to the back of their
/// level instead of to the front (`makeKeyAndOrderFront:` / `orderFront:`),
/// so it never takes focus or covers another app. Opening Finder is skipped.
/// Replace every native panel (open/save) the same way: in background mode
/// read the choice from an env var file instead of showing the panel.
/// The main loop keeps painting while the window is covered.
/// Returns false if a method could not be replaced (then nothing is safe:
/// the caller must not start).
pub fn begin_background() -> bool {
    BACKGROUND.store(true, Ordering::Relaxed);
    #[cfg(target_os = "macos")]
    {
        background_mac::install()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// After launch in background mode: no Dock icon and no Cmd-Tab entry
/// (accessory policy), so nobody switches to the instance by accident.
pub fn background_after_launch() {
    #[cfg(target_os = "macos")]
    if background()
        && let Some(mtm) = objc2::MainThreadMarker::new()
    {
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        // NSApplicationActivationPolicyAccessory = 1.
        // SAFETY: `setActivationPolicy:` takes an NSInteger and returns BOOL.
        let _: objc2::runtime::Bool =
            unsafe { objc2::msg_send![&*app, setActivationPolicy: 1isize] };
        background_mac::count_keys();
    }
}

#[cfg(target_os = "macos")]
mod background_mac {
    use objc2::runtime::{AnyClass, AnyObject, Bool, Imp, Sel};
    use objc2::{msg_send, sel};
    use std::ffi::CStr;

    unsafe extern "C-unwind" fn no_activate_flag(_this: *mut AnyObject, _cmd: Sel, _flag: Bool) {}

    unsafe extern "C-unwind" fn no_activate(_this: *mut AnyObject, _cmd: Sel) {}

    unsafe extern "C-unwind" fn order_back(
        this: *mut AnyObject,
        _cmd: Sel,
        _sender: *mut AnyObject,
    ) {
        // NSWindowBelow (-1) relative to window 0: behind every window of its level.
        // SAFETY: `this` is the NSWindow the replaced method was sent to.
        let _: () = unsafe { msg_send![this, orderWindow: -1isize, relativeTo: 0isize] };
    }

    /// Replaces the implementation of `sel` on `class` (and its subclasses).
    fn replace(class: &CStr, sel: Sel, imp: Imp) -> bool {
        let Some(m) = AnyClass::get(class).and_then(|c| c.instance_method(sel)) else {
            return false;
        };
        // SAFETY: every replacement has the exact signature of the method it replaces.
        unsafe { m.set_implementation(imp) };
        true
    }

    /// A pass-through local monitor that counts key events the input monitor
    /// passed on (play keys are consumed before it and appear in the trace).
    pub fn count_keys() {
        use block2::RcBlock;
        use objc2_app_kit::{NSEvent, NSEventMask};
        use std::ptr::NonNull;
        let block = RcBlock::new(|event: NonNull<NSEvent>| -> *mut NSEvent {
            super::KEY_EVENTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            event.as_ptr()
        });
        // SAFETY: main thread, NSApplication exists; block and token live forever.
        let token = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                NSEventMask::KeyDown | NSEventMask::KeyUp,
                &block,
            )
        };
        std::mem::forget(token);
        std::mem::forget(block);
    }

    pub fn install() -> bool {
        type Flag = unsafe extern "C-unwind" fn(*mut AnyObject, Sel, Bool);
        type Plain = unsafe extern "C-unwind" fn(*mut AnyObject, Sel);
        type Sender = unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject);
        // SAFETY: method implementations are called with the receiver, the
        // selector and the method's arguments; the types above match them.
        let (flag, plain, back) = unsafe {
            (
                std::mem::transmute::<Flag, Imp>(no_activate_flag),
                std::mem::transmute::<Plain, Imp>(no_activate),
                std::mem::transmute::<Sender, Imp>(order_back),
            )
        };
        let mut ok = replace(c"NSApplication", sel!(activateIgnoringOtherApps:), flag);
        // `activate` exists from macOS 14; older systems have nothing to replace.
        if AnyClass::get(c"NSApplication")
            .and_then(|c| c.instance_method(sel!(activate)))
            .is_some()
        {
            ok &= replace(c"NSApplication", sel!(activate), plain);
        }
        ok &= replace(c"NSWindow", sel!(makeKeyAndOrderFront:), back);
        ok &= replace(c"NSWindow", sel!(orderFront:), back);
        ok &= replace(c"NSWindow", sel!(orderFrontRegardless), plain);
        ok
    }
}

/// Opens a file or folder with the default app (Finder for folders).
/// In background mode nothing opens (Finder would come to the front).
pub fn open(path: &Path) {
    if background() {
        eprintln!("background: not opening {}", path.display());
        return;
    }
    let _ = std::process::Command::new("open").arg(path).spawn();
}

/// Holds a latency-critical activity so macOS does not throttle timers or audio.
pub struct NoAppNap {
    #[cfg(target_os = "macos")]
    _token:
        objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn objc2_foundation::NSObjectProtocol>>,
}

impl NoAppNap {
    /// Begins the activity; it ends when the value is dropped.
    pub fn begin() -> NoAppNap {
        #[cfg(target_os = "macos")]
        {
            use objc2_foundation::{NSActivityOptions, NSProcessInfo, NSString};
            let info = NSProcessInfo::processInfo();
            let reason = NSString::from_str("Practice with tight timing");
            let token = info.beginActivityWithOptions_reason(
                NSActivityOptions::UserInitiatedAllowingIdleSystemSleep
                    | NSActivityOptions::LatencyCritical,
                &reason,
            );
            NoAppNap { _token: token }
        }
        #[cfg(not(target_os = "macos"))]
        {
            NoAppNap {}
        }
    }
}

/// Moves a file or folder to the Trash (Finder's "Move to Trash"; nothing is deleted).
pub fn trash(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::{NSFileManager, NSString, NSURL};
        let s = path.to_str().ok_or("path is not UTF-8")?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(s));
        let fm = NSFileManager::defaultManager();
        fm.trashItemAtURL_resultingItemURL_error(&url, None)
            .map_err(|e| e.localizedDescription().to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(format!("no Trash on this platform: {}", path.display()))
    }
}
