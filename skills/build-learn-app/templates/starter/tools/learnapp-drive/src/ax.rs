//! The accessibility tree of one process (egui exports it through AccessKit):
//! list labels, find an element, perform an AX action on it (`AXPress`,
//! `AXScrollToVisible`, `AXIncrement`, …). Nothing here moves the pointer or
//! posts mouse events: actions go to the element, so they work while the app
//! is behind other windows.

use super::cf::{CFArrayGetCount, CFArrayGetTypeID, CFArrayGetValueAtIndex, CFRelease};
use super::cf::{CFBooleanGetTypeID, CFBooleanGetValue};
use super::cf::{CFGetTypeID, CFNumberGetTypeID, CFNumberGetValue, cfstr, cfstring_to_string};
use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CGSize {
    w: f64,
    h: f64,
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
    fn AXUIElementCopyAttributeValue(
        e: *const c_void,
        attr: *const c_void,
        value: *mut *const c_void,
    ) -> i32;
    fn AXUIElementPerformAction(e: *const c_void, action: *const c_void) -> i32;
    fn AXUIElementSetAttributeValue(
        e: *const c_void,
        attr: *const c_void,
        value: *const c_void,
    ) -> i32;
    fn AXUIElementCopyActionNames(e: *const c_void, names: *mut *const c_void) -> i32;
    fn AXValueGetValue(v: *const c_void, ty: u32, out: *mut c_void) -> bool;
    fn AXIsProcessTrusted() -> bool;
}

/// True if this process may use the accessibility API (and post events to pids).
pub fn trusted() -> bool {
    // SAFETY: no arguments.
    unsafe { AXIsProcessTrusted() }
}

/// x, y, w, h in screen points.
pub type Frame = (f64, f64, f64, f64);

fn attr(e: *const c_void, name: &str) -> Option<*const c_void> {
    let key = cfstr(name);
    let mut out: *const c_void = std::ptr::null();
    // SAFETY: valid element and key; the returned value is owned by us.
    let err = unsafe { AXUIElementCopyAttributeValue(e, key, &mut out) };
    // SAFETY: `key` was created above.
    unsafe { CFRelease(key) };
    (err == 0 && !out.is_null()).then_some(out)
}

fn attr_string(e: *const c_void, name: &str) -> String {
    let Some(v) = attr(e, name) else {
        return String::new();
    };
    // SAFETY: `v` is an owned CF object; type-checked before reading.
    unsafe {
        let s = if CFGetTypeID(v) == CFNumberGetTypeID() {
            let mut n: f64 = 0.0;
            // kCFNumberFloat64Type = 6.
            CFNumberGetValue(v, 6, &mut n as *mut f64 as *mut c_void);
            format!("{n}")
        } else if CFGetTypeID(v) == CFBooleanGetTypeID() {
            format!("{}", CFBooleanGetValue(v))
        } else {
            cfstring_to_string(v).unwrap_or_default()
        };
        CFRelease(v);
        s
    }
}

fn frame(e: *const c_void) -> Option<Frame> {
    let pos = attr(e, "AXPosition")?;
    let Some(size) = attr(e, "AXSize") else {
        // SAFETY: `pos` is owned.
        unsafe { CFRelease(pos) };
        return None;
    };
    let mut p = CGPoint::default();
    let mut s = CGSize::default();
    // SAFETY: kAXValueCGPointType = 1, kAXValueCGSizeType = 2.
    unsafe {
        AXValueGetValue(pos, 1, &mut p as *mut _ as *mut c_void);
        AXValueGetValue(size, 2, &mut s as *mut _ as *mut c_void);
        CFRelease(pos);
        CFRelease(size);
    }
    Some((p.x, p.y, s.w, s.h))
}

/// One element of the tree.
pub struct Node {
    /// AXRole.
    pub role: String,
    /// AXTitle.
    pub title: String,
    /// AXDescription.
    pub description: String,
    /// AXValue (strings and numbers).
    pub value: String,
    /// Screen frame.
    pub frame: Option<Frame>,
    /// Path of child indexes from the window list (to find the element again).
    pub path: Vec<usize>,
}

impl Node {
    /// True if the title or the description is `label`.
    pub fn is(&self, label: &str) -> bool {
        self.title == label || self.description == label
    }
}

fn children(e: *const c_void) -> Option<*const c_void> {
    let c = attr(e, "AXChildren")?;
    // SAFETY: type check of an owned CF object.
    if unsafe { CFGetTypeID(c) } == unsafe { CFArrayGetTypeID() } {
        Some(c)
    } else {
        // SAFETY: owned.
        unsafe { CFRelease(c) };
        None
    }
}

fn walk(e: *const c_void, path: &mut Vec<usize>, out: &mut Vec<Node>) {
    if path.len() > 14 || out.len() > 5000 {
        return;
    }
    out.push(Node {
        role: attr_string(e, "AXRole"),
        title: attr_string(e, "AXTitle"),
        description: attr_string(e, "AXDescription"),
        value: attr_string(e, "AXValue"),
        frame: frame(e),
        path: path.clone(),
    });
    if let Some(kids) = children(e) {
        // SAFETY: AXChildren is a CFArray of AXUIElements owned by the array.
        unsafe {
            for i in 0..CFArrayGetCount(kids) {
                path.push(i as usize);
                walk(CFArrayGetValueAtIndex(kids, i), path, out);
                path.pop();
            }
            CFRelease(kids);
        }
    }
}

/// Calls `f` with the window list of `pid` (a CFArray), if any.
fn with_windows<T>(pid: i32, f: impl FnOnce(*const c_void) -> T) -> Option<T> {
    // SAFETY: the application element is owned and released here.
    let app = unsafe { AXUIElementCreateApplication(pid) };
    let out = attr(app, "AXWindows").map(|w| {
        // SAFETY: AXWindows is an owned CFArray.
        let r = if unsafe { CFGetTypeID(w) } == unsafe { CFArrayGetTypeID() } {
            Some(f(w))
        } else {
            None
        };
        // SAFETY: owned.
        unsafe { CFRelease(w) };
        r
    });
    // SAFETY: created above.
    unsafe { CFRelease(app) };
    out.flatten()
}

/// Every element of every window of `pid`. The first query switches the
/// app's AccessKit tree on; until it arrives (a frame or two) only the window
/// is there, so this waits up to 2 s for more than the window.
pub fn nodes(pid: i32) -> Vec<Node> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let n = nodes_once(pid);
        if n.len() > 1 || std::time::Instant::now() > deadline {
            return n;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn nodes_once(pid: i32) -> Vec<Node> {
    with_windows(pid, |windows| {
        let mut out = Vec::new();
        // SAFETY: `windows` is a CFArray of AXUIElements.
        unsafe {
            for i in 0..CFArrayGetCount(windows) {
                let mut path = vec![i as usize];
                walk(CFArrayGetValueAtIndex(windows, i), &mut path, &mut out);
            }
        }
        out
    })
    .unwrap_or_default()
}

/// Number of AX windows of `pid`.
pub fn window_count(pid: i32) -> usize {
    // SAFETY: `w` is a CFArray.
    with_windows(pid, |w| unsafe { CFArrayGetCount(w) } as usize).unwrap_or(0)
}

/// Frame of the first window.
pub fn window_frame(pid: i32) -> Option<Frame> {
    nodes(pid)
        .into_iter()
        .find(|n| n.role == "AXWindow")
        .and_then(|n| n.frame)
}

/// Performs `action` on the element at `path`. Returns the AX error (0 = ok).
pub fn perform(pid: i32, path: &[usize], action: &str) -> i32 {
    // SAFETY: `e` is a live element for the duration of the call.
    at_path(pid, path, |e| unsafe {
        let a = cfstr(action);
        let err = AXUIElementPerformAction(e, a);
        CFRelease(a);
        err
    })
}

/// Sets `AXValue` of the element at `path` to a string (a text field: the
/// app gets an AccessKit SetValue request, no key window needed).
pub fn set_value(pid: i32, path: &[usize], value: &str) -> i32 {
    // SAFETY: `e` is a live element for the duration of the call.
    at_path(pid, path, |e| unsafe {
        let attr = cfstr("AXValue");
        let v = cfstr(value);
        let err = AXUIElementSetAttributeValue(e, attr, v);
        CFRelease(v);
        CFRelease(attr);
        err
    })
}

/// Runs `f` on the element at `path` (window index, then child indices).
fn at_path(pid: i32, path: &[usize], f: impl FnOnce(*const c_void) -> i32) -> i32 {
    with_windows(pid, |windows| {
        // SAFETY: walks owned/borrowed AX elements; each array is released after use.
        unsafe {
            let (&first, rest) = path.split_first()?;
            if first as isize >= CFArrayGetCount(windows) {
                return None;
            }
            let mut e = CFArrayGetValueAtIndex(windows, first as isize);
            let mut held: Vec<*const c_void> = Vec::new();
            for &i in rest {
                let kids = children(e)?;
                held.push(kids);
                if i as isize >= CFArrayGetCount(kids) {
                    return None;
                }
                e = CFArrayGetValueAtIndex(kids, i as isize);
            }
            let err = f(e);
            for h in held {
                CFRelease(h);
            }
            Some(err)
        }
    })
    .flatten()
    .unwrap_or(-25202)
}

/// The action names of the element at `path` (for `labels --actions`).
pub fn actions(pid: i32, path: &[usize]) -> Vec<String> {
    with_windows(pid, |windows| {
        // SAFETY: as in `perform`.
        unsafe {
            let (&first, rest) = path.split_first()?;
            let mut e = CFArrayGetValueAtIndex(windows, first as isize);
            let mut held = Vec::new();
            for &i in rest {
                let kids = children(e)?;
                held.push(kids);
                e = CFArrayGetValueAtIndex(kids, i as isize);
            }
            let mut names: *const c_void = std::ptr::null();
            let mut out = Vec::new();
            if AXUIElementCopyActionNames(e, &mut names) == 0 && !names.is_null() {
                for i in 0..CFArrayGetCount(names) {
                    out.extend(cfstring_to_string(CFArrayGetValueAtIndex(names, i)));
                }
                CFRelease(names);
            }
            for h in held {
                CFRelease(h);
            }
            Some(out)
        }
    })
    .flatten()
    .unwrap_or_default()
}
