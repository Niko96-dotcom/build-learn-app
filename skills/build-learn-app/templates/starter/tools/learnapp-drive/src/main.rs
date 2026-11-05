//! learnapp-drive: real input for the app's verification skill, safe to use
//! while someone works on the same Mac.
//!
//! - Keys are posted to the app process only (`CGEventPostToPid`), never to
//!   the system-wide HID tap: they reach the app's NSEvent monitor with
//!   hardware-style timestamps even while it is behind other apps, and no
//!   other app can receive them. Every key command refuses a pid that is not
//!   a `learnapp` binary.
//! - Controls are pressed through the accessibility API (`AXPress` on the
//!   element with the label); the pointer never moves, no mouse event is posted.
//! - `shot` captures the app window by window id (needs Screen Recording).
//!   In background mode prefer the app's own screenshot (`LEARNAPP_SHOT_DIR`).
//!
//! Needs Accessibility for the calling process (AX and posting to a pid).
//!
//! Commands:
//!   key <name>... --pid P [--gap MS] [--hold MS]   press keys one after another
//!   chord <name>... --pid P [--hold MS]            press together, hold, release
//!   play --pid P --trace FILE [--offset-ms X] [--jitter-ms J] [--skip-every N]
//!        [--extra-every N] [--timeout S] [--seed N]
//!                                    answer the next scored run on time (see `timed`)
//!   labels <pid> [--actions]         role | title | description | value | x,y,w,h
//!   find <pid> <label> [--timeout S] x,y,w,h of the first match
//!   press <pid> <label> [--timeout S] AXPress on the element
//!   action <pid> <label> <AXAction>  any AX action (AXIncrement, AXScrollToVisible, …)
//!   set-value <pid> <label> <text>   AXValue of a text field
//!   window <pid> | window-id <pid>   frame / window-server id of the app window
//!   shot <pid> <file.png>            screenshot of the app window (by window id)
//!   front                            pid and name of the frontmost app
//!   idle                             seconds since the last user input
//!   now                              host time in ns
//!
//! Key names: physical US positions a–z, semicolon, quote, space, return,
//! escape, tab, up, down, left, right, f19, code:0xNN.

mod ax;
mod cf;
mod timed;

use std::ffi::c_void;
use std::time::{Duration, Instant};

/// The app binary name: key and AX commands refuse every other process.
const APP_BINARY: &str = "learnapp";

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceCreate(state: i32) -> *mut c_void;
    fn CGEventCreateKeyboardEvent(source: *mut c_void, keycode: u16, down: bool) -> *mut c_void;
    fn CGEventPostToPid(pid: i32, event: *mut c_void);
    fn CGEventSetFlags(event: *mut c_void, flags: u64);
    fn CGEventSetTimestamp(event: *mut c_void, timestamp: u64);
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> *const c_void;
}

unsafe extern "C" {
    fn proc_pidpath(pid: i32, buf: *mut c_void, size: u32) -> i32;
}

#[repr(C)]
struct Timebase {
    numer: u32,
    denom: u32,
}

unsafe extern "C" {
    fn mach_absolute_time() -> u64;
    fn mach_timebase_info(info: *mut Timebase) -> i32;
}

fn timebase() -> (u128, u128) {
    let mut tb = Timebase { numer: 0, denom: 0 };
    // SAFETY: plain FFI call with a valid pointer.
    unsafe { mach_timebase_info(&mut tb) };
    (tb.numer.max(1) as u128, tb.denom.max(1) as u128)
}

/// Host time in ns (the clock of NSEvent timestamps and of the app's runs).
pub fn now_ns() -> u64 {
    let (n, d) = timebase();
    // SAFETY: no arguments.
    (unsafe { mach_absolute_time() } as u128 * n / d) as u64
}

/// The executable path of a process.
fn exe_path(pid: i32) -> Option<String> {
    let mut buf = vec![0u8; 4096];
    // SAFETY: the buffer is large enough (PROC_PIDPATHINFO_MAXSIZE = 4096).
    let n = unsafe { proc_pidpath(pid, buf.as_mut_ptr() as *mut c_void, buf.len() as u32) };
    (n > 0).then(|| String::from_utf8_lossy(&buf[..n as usize]).into_owned())
}

/// Exits unless `pid` is a running app binary: events never go anywhere else.
pub fn ensure_app(pid: i32) {
    match exe_path(pid) {
        Some(p) if p.ends_with(&format!("/{APP_BINARY}")) => {}
        Some(p) => fail(&format!("pid {pid} is {p}, not {APP_BINARY}: nothing sent")),
        None => fail(&format!("no process {pid}: nothing sent")),
    }
}

/// Posts keyboard events to one process. The event source has a private
/// state, so the user's own modifier keys never mix into these events.
pub struct Poster {
    source: *mut c_void,
    pid: i32,
}

impl Poster {
    /// A poster for app process `pid` (checked).
    pub fn new(pid: i32) -> Poster {
        ensure_app(pid);
        // SAFETY: kCGEventSourceStatePrivate = -1.
        let source = unsafe { CGEventSourceCreate(-1) };
        Poster { source, pid }
    }

    /// Posts now.
    pub fn post(&self, code: u16, down: bool) {
        self.post_stamped(code, down, None);
    }

    /// Posts with an explicit event time (host ns), as a hardware key event
    /// carries its HID timestamp. Events posted to a pid keep a timestamp in
    /// nanoseconds (measured on macOS 27, Apple silicon); None keeps the time
    /// of creation.
    pub fn post_stamped(&self, code: u16, down: bool, host_ns: Option<u64>) {
        // SAFETY: the event is created, posted to the one process and released.
        unsafe {
            let e = CGEventCreateKeyboardEvent(self.source, code, down);
            if e.is_null() {
                eprintln!("CGEventCreateKeyboardEvent failed");
                return;
            }
            if let Some(ns) = host_ns {
                CGEventSetTimestamp(e, ns);
            }
            CGEventSetFlags(e, 0);
            CGEventPostToPid(self.pid, e);
            cf::CFRelease(e);
        }
    }

    /// The first post of a process connects to the window server (~30–60 ms):
    /// press F19 (does nothing in the app) to get that out of the way.
    pub fn warm_up(&self) {
        for down in [true, false] {
            self.post(0x50, down);
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

/// macOS virtual key code of a key name (physical US position).
pub fn keycode(name: &str) -> Option<u16> {
    let n = name.to_lowercase();
    if let Some(hex) = n.strip_prefix("code:0x") {
        return u16::from_str_radix(hex, 16).ok();
    }
    Some(match n.as_str() {
        "a" => 0x00,
        "s" => 0x01,
        "d" => 0x02,
        "f" => 0x03,
        "h" => 0x04,
        "g" => 0x05,
        "z" => 0x06,
        "x" => 0x07,
        "c" => 0x08,
        "v" => 0x09,
        "b" => 0x0B,
        "q" => 0x0C,
        "w" => 0x0D,
        "e" => 0x0E,
        "r" => 0x0F,
        "y" => 0x10,
        "t" => 0x11,
        "o" => 0x1F,
        "u" => 0x20,
        "i" => 0x22,
        "p" => 0x23,
        "l" => 0x25,
        "j" => 0x26,
        "quote" | "'" => 0x27,
        "k" => 0x28,
        "semicolon" | ";" => 0x29,
        "n" => 0x2D,
        "m" => 0x2E,
        "return" | "enter" => 0x24,
        "tab" => 0x30,
        "space" => 0x31,
        "escape" | "esc" => 0x35,
        "f19" => 0x50,
        "left" => 0x7B,
        "right" => 0x7C,
        "down" => 0x7D,
        "up" => 0x7E,
        _ => return None,
    })
}

/// The value after `flag`.
pub fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

/// A numeric flag value.
pub fn num(args: &[String], flag: &str, default: f64) -> f64 {
    arg_value(args, flag)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Flags without a value.
const SWITCHES: [&str; 1] = ["--actions"];

/// Positional arguments (flags and their values removed).
fn names(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
        } else if SWITCHES.contains(&a.as_str()) {
        } else if a.starts_with("--") {
            skip = true;
        } else {
            out.push(a.clone());
        }
    }
    out
}

fn pid_first(rest: &[String]) -> i32 {
    rest.first()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| fail("pid required"))
}

/// `--pid P` (required for anything that sends events).
pub fn pid_flag(rest: &[String]) -> i32 {
    arg_value(rest, "--pid")
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| fail("--pid P (the app pid) is required"))
}

/// Prints `msg` and exits with 2.
pub fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(2);
}

fn codes(rest: &[String]) -> Vec<u16> {
    names(rest)
        .iter()
        .map(|n| keycode(n).unwrap_or_else(|| fail(&format!("unknown key {n}"))))
        .collect()
}

/// Window-server id of the largest normal (layer 0) window of `pid`.
fn window_id(pid: i32) -> Option<i64> {
    // SAFETY: kCGWindowListOptionAll = 0; the array is owned and released here.
    let list = unsafe { CGWindowListCopyWindowInfo(0, 0) };
    if list.is_null() {
        return None;
    }
    let mut best: Option<(f64, i64)> = None;
    // SAFETY: the list is a CFArray of CFDictionaries; bounds is a CFDictionary.
    unsafe {
        for i in 0..cf::CFArrayGetCount(list) {
            let d = cf::CFArrayGetValueAtIndex(list, i);
            if cf::dict_number(d, "kCGWindowOwnerPID") != Some(pid as f64)
                || cf::dict_number(d, "kCGWindowLayer") != Some(0.0)
            {
                continue;
            }
            let k = cf::cfstr("kCGWindowBounds");
            let bounds = cf::CFDictionaryGetValue(d, k);
            cf::CFRelease(k);
            let area = if bounds.is_null() {
                0.0
            } else {
                cf::dict_number(bounds, "Width").unwrap_or(0.0)
                    * cf::dict_number(bounds, "Height").unwrap_or(0.0)
            };
            if let Some(id) = cf::dict_number(d, "kCGWindowNumber")
                && best.is_none_or(|b| area > b.0)
            {
                best = Some((area, id as i64));
            }
        }
        cf::CFRelease(list);
    }
    best.map(|b| b.1)
}

/// The frontmost app: (pid, name), via `lsappinfo`.
pub fn front_app() -> Option<(i32, String)> {
    let front = std::process::Command::new("lsappinfo")
        .arg("front")
        .output()
        .ok()?;
    let asn = String::from_utf8_lossy(&front.stdout).trim().to_string();
    let info = std::process::Command::new("lsappinfo")
        .args(["info", "-only", "pid", &asn])
        .output()
        .ok()?;
    let pid = parse_pid(&String::from_utf8_lossy(&info.stdout))?;
    let name = std::process::Command::new("lsappinfo")
        .args(["info", "-only", "name", &asn])
        .output()
        .ok()
        .map(|o| {
            let t = String::from_utf8_lossy(&o.stdout).to_string();
            t.split('"')
                .nth(3)
                .or(t.split('"').nth(1))
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_default();
    Some((pid, name))
}

/// The number after `pid` in `lsappinfo` output (`pid = 123` or `"pid"=123`).
fn parse_pid(text: &str) -> Option<i32> {
    let rest = &text[text.find("pid")? + 3..];
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Seconds since the last keyboard/mouse input of the user (IOHIDSystem HIDIdleTime).
pub fn idle_s() -> Option<f64> {
    let out = std::process::Command::new("ioreg")
        .args(["-c", "IOHIDSystem", "-d", "4"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.contains("\"HIDIdleTime\""))?;
    let ns: f64 = line.rsplit('=').next()?.trim().parse().ok()?;
    Some(ns / 1e9)
}

fn find_node(pid: i32, label: &str, timeout: f64) -> ax::Node {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout);
    loop {
        let nodes = ax::nodes(pid);
        let win = nodes
            .iter()
            .find(|n| n.role == "AXWindow")
            .and_then(|n| n.frame);
        let mut matches: Vec<ax::Node> = nodes.into_iter().filter(|n| n.is(label)).collect();
        // Prefer an element whose centre lies inside the window.
        matches.sort_by_key(|n| !n.frame.is_some_and(|f| visible_in(win, f)));
        if let Some(n) = matches.into_iter().next() {
            return n;
        }
        if Instant::now() > deadline {
            eprintln!("not found: {label} (list labels: {APP_BINARY}-drive labels {pid})");
            std::process::exit(1);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// True if the centre of `f` lies inside the window.
fn visible_in(win: Option<ax::Frame>, f: ax::Frame) -> bool {
    match win {
        Some((wx, wy, ww, wh)) => {
            let (cx, cy) = (f.0 + f.2 / 2.0, f.1 + f.3 / 2.0);
            f.2 > 0.0 && f.3 > 0.0 && cx > wx && cx < wx + ww && cy > wy && cy < wy + wh
        }
        None => true,
    }
}

fn act(pid: i32, label: &str, action: &str, timeout: f64) {
    ensure_app(pid);
    let n = find_node(pid, label, timeout);
    let err = ax::perform(pid, &n.path, action);
    if err != 0 {
        eprintln!("{action} on {label} failed (AX error {err})");
        std::process::exit(1);
    }
    println!("{action} {label}");
}

fn press_keys(rest: &[String], together: bool, default_hold: f64) {
    let poster = Poster::new(pid_flag(rest));
    let codes = codes(rest);
    poster.warm_up();
    let hold = num(rest, "--hold", default_hold) as u64;
    if together {
        for c in &codes {
            poster.post(*c, true);
            std::thread::sleep(Duration::from_millis(15));
        }
        std::thread::sleep(Duration::from_millis(hold));
        for c in codes.iter().rev() {
            poster.post(*c, false);
        }
    } else {
        let gap = num(rest, "--gap", 120.0) as u64;
        for code in codes {
            poster.post(code, true);
            std::thread::sleep(Duration::from_millis(hold));
            poster.post(code, false);
            std::thread::sleep(Duration::from_millis(gap));
        }
    }
    // Let the app take the last key-up before the source goes away.
    std::thread::sleep(Duration::from_millis(60));
    println!("ok");
}

fn label_arg(rest: &[String], i: usize, what: &str) -> String {
    names(rest)
        .get(i)
        .cloned()
        .unwrap_or_else(|| fail(&format!("{what} required")))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().cloned() else {
        fail(
            "usage: learnapp-drive key|chord|play|labels|find|press|action|set-value|window|window-id|shot|front|idle|now …",
        )
    };
    let rest = &args[1..];
    if matches!(
        cmd.as_str(),
        "key" | "chord" | "play" | "labels" | "find" | "press" | "action" | "set-value"
    ) && !ax::trusted()
    {
        fail(
            "this process lacks Accessibility permission (System Settings → Privacy & Security → Accessibility)",
        );
    }
    match cmd.as_str() {
        "now" => println!("{}", now_ns()),
        "front" => match front_app() {
            Some((pid, name)) => println!("{pid} {name}"),
            None => fail("no front app"),
        },
        "idle" => match idle_s() {
            Some(s) => println!("{s:.1}"),
            None => fail("HIDIdleTime unavailable"),
        },
        "labels" => {
            let pid = pid_first(rest);
            let with_actions = rest.iter().any(|a| a == "--actions");
            for n in ax::nodes(pid) {
                if n.title.is_empty() && n.description.is_empty() && n.value.is_empty() {
                    continue;
                }
                let f = n
                    .frame
                    .map(|(x, y, w, h)| format!("{x:.0},{y:.0},{w:.0},{h:.0}"))
                    .unwrap_or_default();
                let acts = if with_actions {
                    format!(" | {}", ax::actions(pid, &n.path).join(","))
                } else {
                    String::new()
                };
                println!(
                    "{} | {} | {} | {} | {}{}",
                    n.role, n.title, n.description, n.value, f, acts
                );
            }
        }
        "find" => {
            let pid = pid_first(rest);
            let label = label_arg(rest, 1, "label");
            let n = find_node(pid, &label, num(rest, "--timeout", 5.0));
            match n.frame {
                Some((x, y, w, h)) => println!("{x:.0},{y:.0},{w:.0},{h:.0}"),
                None => println!("found {label} (no frame)"),
            }
        }
        "press" => {
            let pid = pid_first(rest);
            let label = label_arg(rest, 1, "label");
            act(pid, &label, "AXPress", num(rest, "--timeout", 5.0));
        }
        "action" => {
            let pid = pid_first(rest);
            let label = label_arg(rest, 1, "label");
            let action = label_arg(rest, 2, "AX action");
            act(pid, &label, &action, num(rest, "--timeout", 5.0));
        }
        "set-value" => {
            let pid = pid_first(rest);
            let label = label_arg(rest, 1, "label");
            let value = label_arg(rest, 2, "value");
            ensure_app(pid);
            let node = find_node(pid, &label, num(rest, "--timeout", 5.0));
            let err = ax::set_value(pid, &node.path, &value);
            if err != 0 {
                eprintln!("set-value on {label} failed (AX error {err})");
                std::process::exit(1);
            }
            println!("AXValue {label} = {value}");
        }
        "window" => match ax::window_frame(pid_first(rest)) {
            Some((x, y, w, h)) => println!("{x:.0},{y:.0},{w:.0},{h:.0}"),
            None => fail("no window"),
        },
        "window-count" => println!("{}", ax::window_count(pid_first(rest))),
        "window-id" => match window_id(pid_first(rest)) {
            Some(id) => println!("{id}"),
            None => fail("no window"),
        },
        "shot" => {
            let pid = pid_first(rest);
            let file = label_arg(rest, 1, "file");
            // The window by id: it may be behind other windows; nothing is raised.
            let Some(id) = window_id(pid) else {
                fail("no window")
            };
            let ok = std::process::Command::new("screencapture")
                .args(["-x", "-o", "-l", &id.to_string(), &file])
                .status()
                .is_ok_and(|s| s.success());
            if !ok {
                fail("screencapture failed (Screen Recording permission?)");
            }
            println!("{file}");
        }
        "key" => press_keys(rest, false, 40.0),
        "chord" => press_keys(rest, true, 600.0),
        "play" => timed::run(rest),
        other => fail(&format!("unknown command {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pids_from_lsappinfo() {
        assert_eq!(
            parse_pid("bundleID=[ NULL ]\n    pid = 69483 !cgs"),
            Some(69483)
        );
        assert_eq!(parse_pid("\"pid\"=42"), Some(42));
        assert_eq!(parse_pid("nothing"), None);
    }

    #[test]
    fn keys_by_physical_name() {
        assert_eq!(keycode("a"), Some(0x00));
        assert_eq!(keycode("F"), Some(0x03));
        assert_eq!(keycode("j"), Some(0x26));
        assert_eq!(keycode("code:0x31"), Some(0x31));
        assert_eq!(keycode("nope"), None);
    }

    #[test]
    fn positional_names_skip_flags() {
        let args: Vec<String> = ["a", "--hold", "300", "--actions", "d"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(names(&args), vec!["a", "d"]);
        assert_eq!(num(&args, "--hold", 1.0), 300.0);
    }

    #[test]
    fn visibility_needs_the_centre_inside_the_window() {
        let win = Some((0.0, 0.0, 100.0, 100.0));
        assert!(visible_in(win, (10.0, 40.0, 10.0, 10.0)));
        assert!(!visible_in(win, (10.0, 120.0, 10.0, 10.0)));
    }

    #[test]
    fn only_the_app_gets_events() {
        assert!(exe_path(std::process::id() as i32).is_some_and(|p| !p.ends_with("/learnapp")));
    }
}
