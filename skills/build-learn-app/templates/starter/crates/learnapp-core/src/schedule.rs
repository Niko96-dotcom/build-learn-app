//! Review scheduling: retention, not same-session fluency, decides "learned".
//!
//! An item moves New → Learning → ColdPending → Known. A lesson passed today
//! makes it ColdPending; only a clean cold run on a *later* day makes it
//! Known. Known items have a half-life h (days): predicted recall after a gap
//! g is 2^(−g/h), and the item is due when recall drops below
//! [`DUE_BELOW`]. A pass after a real gap lengthens h by the gap; an early
//! review changes nothing; a failure halves h (never below one day) and
//! never resets the item to zero.

use serde::{Deserialize, Serialize};

/// An item is due when predicted recall falls below this.
pub const DUE_BELOW: f64 = 0.85;

/// Where an item stands.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    /// Never practised.
    #[default]
    New,
    /// Practised, lesson not passed yet.
    Learning,
    /// Lesson passed; waits for a cold run on a later day.
    ColdPending,
    /// Passed a cold run on a later day; reviewed on a spaced schedule.
    Known,
}

/// What happened in one run of an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Met the gate of the item.
    Pass,
    /// Missed the gate.
    Fail,
}

/// Review memory of one item.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemMemory {
    /// Current stage.
    pub stage: Stage,
    /// Half-life in days (Known items).
    pub half_life: f64,
    /// Day of the last run.
    pub last_day: i64,
    /// Failed reviews of a Known item.
    pub lapses: u32,
}

impl Default for ItemMemory {
    fn default() -> Self {
        ItemMemory {
            stage: Stage::New,
            half_life: 1.0,
            last_day: i64::MIN,
            lapses: 0,
        }
    }
}

impl ItemMemory {
    /// Predicted recall on `today` (1.0 for items that are not Known yet).
    pub fn recall(&self, today: i64) -> f64 {
        if self.stage != Stage::Known {
            return 1.0;
        }
        let gap = (today - self.last_day).max(0) as f64;
        0.5f64.powf(gap / self.half_life.max(1.0))
    }

    /// True if the item should come up today.
    pub fn is_due(&self, today: i64) -> bool {
        match self.stage {
            Stage::New | Stage::Learning => false,
            // A cold run is only cold on a later day.
            Stage::ColdPending => today > self.last_day,
            Stage::Known => self.recall(today) < DUE_BELOW,
        }
    }

    /// The result of a lesson (guided practice) on `today`.
    pub fn lesson(&mut self, today: i64, outcome: Outcome) {
        if self.stage == Stage::Known {
            // Extra practice on a known item is not a review.
            return;
        }
        self.stage = match outcome {
            Outcome::Pass => Stage::ColdPending,
            Outcome::Fail => Stage::Learning,
        };
        self.last_day = today;
    }

    /// The result of a cold run (no guide, no warm-up on the item) on `today`.
    /// Returns false (and changes nothing) for a run that cannot count as cold.
    pub fn cold(&mut self, today: i64, outcome: Outcome) -> bool {
        match self.stage {
            Stage::ColdPending if today > self.last_day => {
                match outcome {
                    Outcome::Pass => {
                        self.stage = Stage::Known;
                        self.half_life = 1.0;
                    }
                    Outcome::Fail => self.stage = Stage::Learning,
                }
                self.last_day = today;
                true
            }
            Stage::Known => {
                let gap = (today - self.last_day).max(0) as f64;
                match outcome {
                    // An early review does not lengthen the half-life.
                    Outcome::Pass if self.is_due(today) => self.half_life += gap,
                    Outcome::Pass => {}
                    Outcome::Fail => {
                        self.half_life = (self.half_life * 0.5).max(1.0);
                        self.lapses += 1;
                    }
                }
                self.last_day = today;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_day_pass_is_not_learned() {
        let mut m = ItemMemory::default();
        m.lesson(10, Outcome::Pass);
        assert_eq!(m.stage, Stage::ColdPending);
        assert!(!m.is_due(10));
        assert!(!m.cold(10, Outcome::Pass), "a same-day run is not cold");
        assert_eq!(m.stage, Stage::ColdPending);
        assert!(m.is_due(11));
        assert!(m.cold(11, Outcome::Pass));
        assert_eq!(m.stage, Stage::Known);
    }

    #[test]
    fn failed_cold_run_goes_back_to_learning() {
        let mut m = ItemMemory::default();
        m.lesson(1, Outcome::Pass);
        assert!(m.cold(2, Outcome::Fail));
        assert_eq!(m.stage, Stage::Learning);
    }

    #[test]
    fn spacing_grows_and_lapses_halve() {
        let mut m = ItemMemory::default();
        m.lesson(0, Outcome::Pass);
        m.cold(1, Outcome::Pass);
        // h = 1: recall after 1 day = 0.5 < 0.85 → due next day.
        assert!(m.is_due(2));
        m.cold(2, Outcome::Pass);
        assert_eq!(m.half_life, 2.0);
        assert!(!m.is_due(2));
        m.cold(4, Outcome::Fail);
        assert_eq!(m.half_life, 1.0);
        assert_eq!(m.lapses, 1);
        assert_eq!(m.stage, Stage::Known, "a lapse never resets to zero");
    }

    #[test]
    fn early_review_does_not_lengthen() {
        let mut m = ItemMemory {
            stage: Stage::Known,
            half_life: 8.0,
            last_day: 0,
            lapses: 0,
        };
        assert!(!m.is_due(1));
        m.cold(1, Outcome::Pass);
        assert_eq!(m.half_life, 8.0);
    }

    #[test]
    fn extra_lesson_on_known_item_changes_nothing() {
        let mut m = ItemMemory {
            stage: Stage::Known,
            half_life: 4.0,
            last_day: 3,
            lapses: 0,
        };
        let before = m;
        m.lesson(5, Outcome::Fail);
        assert_eq!(m, before);
    }
}
