//! `play`: answer the next scored run on time, like a learner would.
//!
//! The app publishes every scored run as a trace event
//! `{"kind":"transport","data":{"scored":true,"zero_host_ns":…,"targets":[{"ms":…,"key":"f","hold_ms":80},…]}}`.
//! `play` notes the trace size, waits for the next such event, then posts
//! every target key at `zero_host_ns + ms + offset (± jitter)` with the
//! planned time as the event timestamp. Options shape the answer:
//! `--offset-ms X` plays every press X ms late (negative = early),
//! `--jitter-ms J` adds Gaussian jitter (seeded), `--skip-every N` leaves out
//! every Nth target, `--extra-every N` presses the same key a second time
//! 150 ms after every Nth target (an extra hit outside every window). It prints one JSON line: sent, skipped, extras, late (posts
//! that left more than 2 ms after their plan; must be 0), worst_late_ms.

use crate::{Poster, arg_value, fail, keycode, now_ns, num, pid_flag};
use std::io::{Read, Seek, SeekFrom};
use std::time::{Duration, Instant};

/// One planned post.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Post {
    at_ns: u64,
    code: u16,
    down: bool,
}

/// A small seeded generator (xorshift) for jitter.
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Standard normal (Box–Muller).
    fn gauss(&mut self) -> f64 {
        let u = self.next_f64().max(1e-12);
        let v = self.next_f64();
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }
}

/// Options of one `play`.
struct Shape {
    offset_ms: f64,
    jitter_ms: f64,
    skip_every: usize,
    extra_every: usize,
    seed: u64,
}

/// The planned posts for a transport event.
fn plan(transport: &serde_json::Value, s: &Shape) -> (Vec<Post>, usize, usize) {
    let zero = transport["zero_host_ns"]
        .as_u64()
        .unwrap_or_else(|| fail("transport without zero_host_ns"));
    let targets = transport["targets"].as_array().cloned().unwrap_or_default();
    let mut rng = Rng(s.seed.max(1));
    let mut posts = Vec::new();
    let (mut skipped, mut extras) = (0, 0);
    for (i, t) in targets.iter().enumerate() {
        let n = i + 1;
        if s.skip_every > 0 && n % s.skip_every == 0 {
            skipped += 1;
            continue;
        }
        let code = t["code"]
            .as_u64()
            .map(|c| c as u16)
            .or_else(|| t["key"].as_str().and_then(keycode))
            .unwrap_or_else(|| fail(&format!("target {i} has no key")));
        let ms = t["ms"].as_f64().unwrap_or(0.0) + s.offset_ms + s.jitter_ms * rng.gauss();
        let hold = t["hold_ms"].as_f64().unwrap_or(80.0);
        let at = (zero as f64 + ms * 1e6).max(0.0) as u64;
        posts.push(Post {
            at_ns: at,
            code,
            down: true,
        });
        posts.push(Post {
            at_ns: at + (hold * 1e6) as u64,
            code,
            down: false,
        });
        if s.extra_every > 0 && n % s.extra_every == 0 {
            // The same key again after the first press is released: a double press.
            let again = at + ((hold + 70.0).max(150.0) * 1e6) as u64;
            posts.push(Post {
                at_ns: again,
                code,
                down: true,
            });
            posts.push(Post {
                at_ns: again + 40_000_000,
                code,
                down: false,
            });
            extras += 1;
        }
    }
    posts.sort_by_key(|p| p.at_ns);
    (posts, skipped, extras)
}

/// Sleeps until `target_ns`, then spins (macOS wakes sleeping threads a few ms late).
fn wait_until(target_ns: u64) {
    loop {
        let n = now_ns();
        if n >= target_ns {
            return;
        }
        let left = target_ns - n;
        if left > 8_000_000 {
            std::thread::sleep(Duration::from_nanos(left - 6_000_000));
        } else {
            std::hint::spin_loop();
        }
    }
}

/// Waits for the next scored `transport` event appended after byte `from`.
fn wait_transport(path: &std::path::Path, from: u64, timeout: f64) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout);
    let mut pos = from;
    let mut buf = String::new();
    loop {
        if let Ok(mut f) = std::fs::File::open(path)
            && f.seek(SeekFrom::Start(pos)).is_ok()
        {
            let mut chunk = String::new();
            if f.read_to_string(&mut chunk).is_ok() {
                pos += chunk.len() as u64;
                buf.push_str(&chunk);
                while let Some(nl) = buf.find('\n') {
                    let line: String = buf.drain(..=nl).collect();
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line)
                        && v["kind"] == "transport"
                        && v["data"]["scored"] == true
                    {
                        return v["data"].clone();
                    }
                }
            }
        }
        if Instant::now() > deadline {
            fail("timeout: no scored run started (press Start after play is waiting)");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// `play --pid P --trace FILE …`.
pub fn run(rest: &[String]) {
    let pid = pid_flag(rest);
    let trace = arg_value(rest, "--trace").unwrap_or_else(|| fail("--trace FILE required"));
    let path = std::path::PathBuf::from(&trace);
    let from = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let poster = Poster::new(pid);
    let shape = Shape {
        offset_ms: num(rest, "--offset-ms", 0.0),
        jitter_ms: num(rest, "--jitter-ms", 0.0),
        skip_every: num(rest, "--skip-every", 0.0) as usize,
        extra_every: num(rest, "--extra-every", 0.0) as usize,
        seed: num(rest, "--seed", 7.0) as u64,
    };
    eprintln!("play: waiting for the next scored run in {trace}");
    let transport = wait_transport(&path, from, num(rest, "--timeout", 30.0));
    let (posts, skipped, extras) = plan(&transport, &shape);
    // Warm up the window-server connection well before the first press.
    if let Some(first) = posts.first() {
        wait_until(first.at_ns.saturating_sub(120_000_000));
        poster.warm_up();
    }
    let (mut late, mut worst) = (0usize, 0.0f64);
    for p in &posts {
        wait_until(p.at_ns);
        let lag_ms = (now_ns().saturating_sub(p.at_ns)) as f64 / 1e6;
        if lag_ms > 2.0 {
            late += 1;
        }
        worst = worst.max(lag_ms);
        poster.post_stamped(p.code, p.down, Some(p.at_ns));
    }
    std::thread::sleep(Duration::from_millis(60));
    println!(
        "{}",
        serde_json::json!({
            "sent": posts.iter().filter(|p| p.down).count(),
            "skipped": skipped,
            "extras": extras,
            "late": late,
            "worst_late_ms": (worst * 100.0).round() / 100.0,
        })
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transport() -> serde_json::Value {
        serde_json::json!({
            "zero_host_ns": 1_000_000_000u64,
            "targets": [
                {"ms": 0.0, "key": "f", "hold_ms": 80},
                {"ms": 600.0, "code": 38, "hold_ms": 80},
                {"ms": 1200.0, "key": "f"},
                {"ms": 1800.0, "key": "j"},
            ]
        })
    }

    fn shape() -> Shape {
        Shape {
            offset_ms: 0.0,
            jitter_ms: 0.0,
            skip_every: 0,
            extra_every: 0,
            seed: 1,
        }
    }

    #[test]
    fn plans_downs_and_ups_on_time() {
        let (posts, skipped, extras) = plan(&transport(), &shape());
        assert_eq!((skipped, extras), (0, 0));
        assert_eq!(posts.len(), 8);
        assert_eq!(
            posts[0],
            Post {
                at_ns: 1_000_000_000,
                code: 0x03,
                down: true
            }
        );
        assert_eq!(
            posts[1],
            Post {
                at_ns: 1_080_000_000,
                code: 0x03,
                down: false
            }
        );
        assert!(
            posts
                .iter()
                .any(|p| p.code == 38 && p.at_ns == 1_600_000_000)
        );
    }

    #[test]
    fn offset_skip_and_extra() {
        let s = Shape {
            offset_ms: 20.0,
            skip_every: 2,
            extra_every: 1,
            ..shape()
        };
        let (posts, skipped, extras) = plan(&transport(), &s);
        assert_eq!((skipped, extras), (2, 2));
        let downs: Vec<_> = posts.iter().filter(|p| p.down).collect();
        assert_eq!(downs.len(), 4, "two targets, two extras");
        assert_eq!(downs[0].at_ns, 1_020_000_000);
        assert_eq!(
            downs[1].at_ns, 1_170_000_000,
            "extra 150 ms after the first"
        );
    }

    #[test]
    fn jitter_is_seeded() {
        let s = Shape {
            jitter_ms: 10.0,
            ..shape()
        };
        assert_eq!(plan(&transport(), &s).0, plan(&transport(), &s).0);
    }
}
