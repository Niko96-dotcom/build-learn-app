//! Day numbers (days since 1970-01-01 in local time) and their names.
//! The core crate never reads a clock: the app passes the day in, so tests
//! and verification can use a test clock (`--now`).

/// Days since 1970-01-01 for a civil date (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = m as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date (year, month 1..12, day 1..31) of a day number.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

/// Parses `YYYY-MM-DD` (a time part after `T` is ignored).
pub fn parse_day(s: &str) -> Option<i64> {
    let date = s.split('T').next()?;
    let mut it = date.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: u32 = it.next()?.parse().ok()?;
    let d: u32 = it.next()?.parse().ok()?;
    if it.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let day = days_from_civil(y, m, d);
    (civil_from_days(day) == (y, m, d)).then_some(day)
}

/// Today's day number in local time.
pub fn today_local() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    (secs + local_offset_s(secs)).div_euclid(86_400)
}

#[cfg(unix)]
fn local_offset_s(secs: i64) -> i64 {
    #[repr(C)]
    struct Tm {
        sec: i32,
        min: i32,
        hour: i32,
        mday: i32,
        mon: i32,
        year: i32,
        wday: i32,
        yday: i32,
        isdst: i32,
        gmtoff: std::ffi::c_long,
        zone: *const std::ffi::c_char,
    }
    unsafe extern "C" {
        fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
    }
    let mut tm = Tm {
        sec: 0,
        min: 0,
        hour: 0,
        mday: 0,
        mon: 0,
        year: 0,
        wday: 0,
        yday: 0,
        isdst: 0,
        gmtoff: 0,
        zone: std::ptr::null(),
    };
    // SAFETY: plain libc call with valid pointers; `tm` is fully written on success.
    let ok = unsafe { !localtime_r(&secs, &mut tm).is_null() };
    if ok { tm.gmtoff as i64 } else { 0 }
}

#[cfg(not(unix))]
fn local_offset_s(_secs: i64) -> i64 {
    0
}

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// "Thursday 5 November".
pub fn long_name(day: i64) -> String {
    // 1970-01-01 was a Thursday (index 3 from Monday).
    let wd = WEEKDAYS[(day + 3).rem_euclid(7) as usize];
    let (_, m, d) = civil_from_days(day);
    format!("{wd} {d} {}", MONTHS[(m - 1) as usize])
}

/// "5 Nov".
pub fn short_name(day: i64) -> String {
    let (_, m, d) = civil_from_days(day);
    format!("{d} {}", &MONTHS[(m - 1) as usize][..3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for day in [-800_000, -1, 0, 1, 20_397, 400_000] {
            let (y, m, d) = civil_from_days(day);
            assert_eq!(days_from_civil(y, m, d), day);
        }
        assert_eq!(days_from_civil(1970, 1, 1), 0);
    }

    #[test]
    fn parse_and_names() {
        let day = parse_day("2026-11-05").unwrap();
        assert_eq!(long_name(day), "Thursday 5 November");
        assert_eq!(short_name(day), "5 Nov");
        assert_eq!(parse_day("2026-11-05T09:30"), Some(day));
        assert_eq!(parse_day("2026-02-30"), None);
        assert_eq!(parse_day("nope"), None);
    }
}
