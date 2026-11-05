//! The learner profile: settings, practice days and per-item memory.
//! Progress is measured skill, never points for time spent.

use crate::schedule::ItemMemory;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Schema version of `profile.json`. A newer file is never overwritten.
pub const PROFILE_VERSION: u32 = 1;

/// Days per week the learner aims for. A missed day never shows as a loss.
pub const WEEKLY_TARGET: usize = 5;

/// Everything the app remembers about one learner.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Schema version (see [`PROFILE_VERSION`]).
    pub version: u32,
    /// Day number (days since 1970-01-01, local) of the first start.
    pub created_day: i64,
    /// Learner settings.
    #[serde(default)]
    pub settings: Settings,
    /// Days with at least one completed session.
    #[serde(default)]
    pub practice_days: BTreeSet<i64>,
    /// Finished runs (all kinds).
    #[serde(default)]
    pub runs: u32,
    /// Review memory per item id.
    #[serde(default)]
    pub items: BTreeMap<String, ItemMemory>,
}

/// Learner settings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    /// Fewer and shorter animations.
    #[serde(default)]
    pub reduce_motion: bool,
    /// Output level 0..1.
    #[serde(default = "default_level")]
    pub level: f32,
}

fn default_level() -> f32 {
    0.8
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            reduce_motion: false,
            level: default_level(),
        }
    }
}

impl Profile {
    /// A fresh profile created on `day`.
    pub fn new(day: i64) -> Profile {
        Profile {
            version: PROFILE_VERSION,
            created_day: day,
            settings: Settings::default(),
            practice_days: BTreeSet::new(),
            runs: 0,
            items: BTreeMap::new(),
        }
    }

    /// The last seven days, oldest first: true where the learner practised.
    pub fn week_ring(&self, today: i64) -> [bool; 7] {
        let mut ring = [false; 7];
        for (i, slot) in ring.iter_mut().enumerate() {
            *slot = self.practice_days.contains(&(today - 6 + i as i64));
        }
        ring
    }

    /// Practice days in the last seven days.
    pub fn days_this_week(&self, today: i64) -> usize {
        self.week_ring(today).iter().filter(|d| **d).count()
    }

    /// Items whose review is due on `today`, most urgent first.
    pub fn due(&self, today: i64) -> Vec<&str> {
        let mut due: Vec<(&str, f64)> = self
            .items
            .iter()
            .filter(|(_, m)| m.is_due(today))
            .map(|(id, m)| (id.as_str(), m.recall(today)))
            .collect();
        due.sort_by(|a, b| a.1.total_cmp(&b.1));
        due.into_iter().map(|(id, _)| id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_counts_the_last_seven_days() {
        let mut p = Profile::new(100);
        p.practice_days.extend([94, 99, 100, 101]);
        assert_eq!(
            p.week_ring(100),
            [true, false, false, false, false, true, true]
        );
        assert_eq!(p.days_this_week(100), 3);
    }

    #[test]
    fn old_files_get_defaults() {
        let p: Profile = serde_json::from_str(r#"{"version":1,"created_day":5}"#).unwrap();
        assert_eq!(p.settings, Settings::default());
        assert!(p.items.is_empty());
    }
}
