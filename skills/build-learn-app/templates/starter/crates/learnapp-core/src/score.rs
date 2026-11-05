//! Timing scores for a run: each hit is matched to at most one target of the
//! same key, nearest first, inside the counted window. The summary states
//! measured facts (accuracy, bias, spread, misses, extras) and the verdict
//! never says "clean" when anything was missed or extra.

use serde::{Deserialize, Serialize};

/// One planned key press, in ms from the run's zero.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Target {
    /// Time in ms after the run's zero.
    pub ms: f64,
    /// macOS virtual key code (physical position).
    pub key: u16,
}

/// One key press of the learner, in ms from the run's zero (calibrated).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Hit {
    /// Time in ms after the run's zero.
    pub ms: f64,
    /// macOS virtual key code.
    pub key: u16,
}

/// Hit windows in ± ms. They tighten with the learner's stage.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Windows {
    /// "Perfect".
    pub perfect: f64,
    /// "Good": only hits outside this window are marked during a run.
    pub good: f64,
    /// Anything farther is a miss plus an extra.
    pub counted: f64,
}

impl Windows {
    /// Beginner windows (35 / 60 / 90 ms).
    pub const BEGINNER: Windows = Windows {
        perfect: 35.0,
        good: 60.0,
        counted: 90.0,
    };

    /// Never tighter than three times the learner's measured jitter.
    pub fn with_jitter_floor(self, jitter_ms: f64) -> Windows {
        let floor = 3.0 * jitter_ms.max(0.0);
        Windows {
            perfect: self.perfect.max(floor),
            good: self.good.max(floor),
            counted: self.counted.max(floor),
        }
    }
}

/// The measured result of one run.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Summary {
    /// Planned presses.
    pub targets: usize,
    /// Targets hit inside the counted window.
    pub matched: usize,
    /// Matched inside the perfect window.
    pub perfect: usize,
    /// Matched inside the good window (perfect included).
    pub good: usize,
    /// Targets without a hit.
    pub missed: usize,
    /// Hits without a target.
    pub extras: usize,
    /// matched ÷ (targets + 0.5 × extras), 0..1.
    pub accuracy: f64,
    /// Mean signed offset of matched hits (ms, negative = early).
    pub bias_ms: f64,
    /// Standard deviation of the offsets (ms).
    pub spread_ms: f64,
    /// Offset per target (None = missed), in target order.
    pub offsets: Vec<Option<f64>>,
}

/// Scores `hits` against `targets`.
pub fn score(targets: &[Target], hits: &[Hit], w: Windows) -> Summary {
    let mut offsets: Vec<Option<f64>> = vec![None; targets.len()];
    let mut used = vec![false; hits.len()];
    // All candidate pairs, closest first: a greedy one-to-one match.
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for (ti, t) in targets.iter().enumerate() {
        for (hi, h) in hits.iter().enumerate() {
            let d = h.ms - t.ms;
            if h.key == t.key && d.abs() <= w.counted {
                pairs.push((d.abs(), ti, hi));
            }
        }
    }
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, ti, hi) in pairs {
        if offsets[ti].is_none() && !used[hi] {
            offsets[ti] = Some(hits[hi].ms - targets[ti].ms);
            used[hi] = true;
        }
    }
    let matched: Vec<f64> = offsets.iter().flatten().copied().collect();
    let n = matched.len();
    let extras = used.iter().filter(|u| !**u).count();
    let bias = if n > 0 {
        matched.iter().sum::<f64>() / n as f64
    } else {
        0.0
    };
    let spread = if n > 1 {
        (matched.iter().map(|o| (o - bias).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    } else {
        0.0
    };
    let denom = targets.len() as f64 + 0.5 * extras as f64;
    Summary {
        targets: targets.len(),
        matched: n,
        perfect: matched.iter().filter(|o| o.abs() <= w.perfect).count(),
        good: matched.iter().filter(|o| o.abs() <= w.good).count(),
        missed: targets.len() - n,
        extras,
        accuracy: if denom > 0.0 { n as f64 / denom } else { 0.0 },
        bias_ms: bias,
        spread_ms: spread,
        offsets,
    }
}

/// A gate: the bar a run must clear.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gate {
    /// Minimum accuracy 0..1.
    pub accuracy: f64,
    /// Maximum spread in ms.
    pub spread_ms: f64,
}

impl Gate {
    /// Beginner gate: 85 % accuracy, spread ≤ 40 ms.
    pub const BEGINNER: Gate = Gate {
        accuracy: 0.85,
        spread_ms: 40.0,
    };

    /// True if `s` clears the gate.
    pub fn passed(&self, s: &Summary) -> bool {
        s.targets > 0 && s.accuracy >= self.accuracy && s.spread_ms <= self.spread_ms
    }
}

/// One honest line about the run, built only from measured numbers.
pub fn verdict(s: &Summary, gate: Gate) -> String {
    if s.targets == 0 {
        return "Nothing to score.".into();
    }
    let mut parts = vec![format!("{} of {} in time", s.matched, s.targets)];
    if s.missed > 0 {
        parts.push(format!("{} missed", s.missed));
    }
    if s.extras > 0 {
        let word = if s.extras == 1 {
            "extra hit"
        } else {
            "extra hits"
        };
        parts.push(format!("{} {word}", s.extras));
    }
    let facts = parts.join(", ");
    if gate.passed(s) {
        format!("{facts}. Passed.")
    } else if s.matched > 1 && s.spread_ms > gate.spread_ms {
        format!(
            "{facts}. Spread {:.0} ms: aim for {:.0} ms or less.",
            s.spread_ms, gate.spread_ms
        )
    } else {
        format!("{facts}. Again?")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(ms: f64) -> Target {
        Target { ms, key: 3 }
    }

    fn h(ms: f64) -> Hit {
        Hit { ms, key: 3 }
    }

    #[test]
    fn on_time_run() {
        let targets = [t(0.0), t(600.0), t(1200.0), t(1800.0)];
        let hits = [h(10.0), h(610.0), h(1210.0), h(1810.0)];
        let s = score(&targets, &hits, Windows::BEGINNER);
        assert_eq!((s.matched, s.missed, s.extras), (4, 0, 0));
        assert!((s.bias_ms - 10.0).abs() < 1e-9);
        assert!(s.spread_ms.abs() < 1e-9);
        assert_eq!(verdict(&s, Gate::BEGINNER), "4 of 4 in time. Passed.");
    }

    #[test]
    fn one_hit_matches_one_target() {
        // Two targets 50 ms apart, one hit between them: one match, one miss.
        let s = score(&[t(0.0), t(50.0)], &[h(20.0)], Windows::BEGINNER);
        assert_eq!((s.matched, s.missed, s.extras), (1, 1, 0));
    }

    #[test]
    fn wrong_key_is_extra_not_a_match() {
        let s = score(&[t(0.0)], &[Hit { ms: 0.0, key: 9 }], Windows::BEGINNER);
        assert_eq!((s.matched, s.missed, s.extras), (0, 1, 1));
        assert_eq!(s.accuracy, 0.0);
        assert_eq!(
            verdict(&s, Gate::BEGINNER),
            "0 of 1 in time, 1 missed, 1 extra hit. Again?"
        );
    }

    #[test]
    fn verdict_never_hides_a_miss() {
        let targets: Vec<Target> = (0..8).map(|i| t(i as f64 * 500.0)).collect();
        let hits: Vec<Hit> = (0..7).map(|i| h(i as f64 * 500.0)).collect();
        let s = score(&targets, &hits, Windows::BEGINNER);
        let v = verdict(&s, Gate::BEGINNER);
        assert!(v.starts_with("7 of 8 in time, 1 missed"), "{v}");
    }

    #[test]
    fn jitter_floor_widens_windows() {
        let w = Windows::BEGINNER.with_jitter_floor(20.0);
        assert_eq!((w.perfect, w.good, w.counted), (60.0, 60.0, 90.0));
    }

    #[test]
    fn far_hits_do_not_count() {
        let s = score(&[t(0.0)], &[h(91.0)], Windows::BEGINNER);
        assert_eq!((s.matched, s.extras), (0, 1));
    }
}
