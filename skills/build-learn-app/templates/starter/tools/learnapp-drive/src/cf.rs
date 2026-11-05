//! The CoreFoundation calls the driver needs.

use std::ffi::{c_char, c_void};

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub fn CFRelease(cf: *const c_void);
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        s: *const c_char,
        encoding: u32,
    ) -> *const c_void;
    fn CFStringGetCString(s: *const c_void, buf: *mut c_char, len: isize, encoding: u32) -> bool;
    pub fn CFGetTypeID(cf: *const c_void) -> usize;
    fn CFStringGetTypeID() -> usize;
    pub fn CFArrayGetTypeID() -> usize;
    pub fn CFNumberGetTypeID() -> usize;
    pub fn CFArrayGetCount(a: *const c_void) -> isize;
    pub fn CFArrayGetValueAtIndex(a: *const c_void, i: isize) -> *const c_void;
    pub fn CFDictionaryGetValue(d: *const c_void, key: *const c_void) -> *const c_void;
    pub fn CFNumberGetValue(n: *const c_void, ty: isize, out: *mut c_void) -> bool;
    pub fn CFBooleanGetTypeID() -> usize;
    pub fn CFBooleanGetValue(b: *const c_void) -> bool;
}

const UTF8: u32 = 0x0800_0100;

/// An owned CFString (release it).
pub fn cfstr(s: &str) -> *const c_void {
    let c = std::ffi::CString::new(s).unwrap_or_default();
    // SAFETY: creates an owned CFString; callers release it.
    unsafe { CFStringCreateWithCString(std::ptr::null(), c.as_ptr(), UTF8) }
}

/// The text of a CFString (None for other types).
///
/// # Safety
/// `v` must be a valid CF object.
pub unsafe fn cfstring_to_string(v: *const c_void) -> Option<String> {
    // SAFETY: type-checked before reading.
    unsafe {
        if v.is_null() || CFGetTypeID(v) != CFStringGetTypeID() {
            return None;
        }
        let mut buf = vec![0 as c_char; 2048];
        CFStringGetCString(v, buf.as_mut_ptr(), buf.len() as isize, UTF8).then(|| {
            std::ffi::CStr::from_ptr(buf.as_ptr())
                .to_string_lossy()
                .into_owned()
        })
    }
}

/// A CFNumber value of a CFDictionary as f64.
pub fn dict_number(d: *const c_void, key: &str) -> Option<f64> {
    let k = cfstr(key);
    // SAFETY: `d` is a CFDictionary; the value is borrowed from it.
    let v = unsafe { CFDictionaryGetValue(d, k) };
    // SAFETY: `k` was created above.
    unsafe { CFRelease(k) };
    let mut out: f64 = 0.0;
    // SAFETY: kCFNumberFloat64Type = 6, `out` is an f64.
    (!v.is_null() && unsafe { CFNumberGetValue(v, 6, &mut out as *mut f64 as *mut c_void) })
        .then_some(out)
}
