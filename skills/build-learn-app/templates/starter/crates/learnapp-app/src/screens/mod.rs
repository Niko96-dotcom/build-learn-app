//! Screens. Each screen is a function `show(ui, content, frame, view) ->
//! Option<Action>`: it paints a view model and returns what the learner
//! asked for. The app applies the action. Screens never touch the store.

pub mod practice;
pub mod progress;
pub mod settings;
pub mod today;

use crate::theme;

/// The screens of the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// The daily plan.
    Today,
    /// A scored run.
    Practice,
    /// Measured progress.
    Progress,
    /// Settings.
    Settings,
}

impl Screen {
    /// Top navigation: screen and visible label (also its accessibility label).
    pub const NAV: [(Screen, &'static str); 4] = [
        (Screen::Today, "Today"),
        (Screen::Practice, "Practice"),
        (Screen::Progress, "Progress"),
        (Screen::Settings, "Settings"),
    ];

    /// Stable id for `state.json`, `--screen` and the trace.
    pub fn id(self) -> &'static str {
        match self {
            Screen::Today => "today",
            Screen::Practice => "practice",
            Screen::Progress => "progress",
            Screen::Settings => "settings",
        }
    }

    /// The screen with this id.
    pub fn parse(id: &str) -> Option<Screen> {
        Self::NAV.iter().map(|(s, _)| *s).find(|s| s.id() == id)
    }
}

/// What the learner asked for on a screen.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Open a screen.
    Go(Screen),
    /// Start a scored run of the current item.
    Start,
    /// Leave the run (it ends; nothing is scored).
    Leave,
    /// Settings: reduce motion on or off.
    ReduceMotion(bool),
    /// Settings: output level.
    Level(f32),
    /// Settings: first click of "Reset progress".
    ResetAsk,
    /// Settings: second click, moves the profile to the Trash.
    ResetConfirm,
    /// Settings: keep the profile.
    ResetCancel,
}

/// Animation clock of the current frame.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    /// Seconds since the app started.
    pub t: f64,
    /// When the current screen opened.
    pub since: f64,
    /// Fewer and shorter animations.
    pub reduce_motion: bool,
}

impl Frame {
    /// 0..1 enter animation of the screen (0.32 s ease-out).
    pub fn enter(&self) -> f32 {
        if self.reduce_motion {
            return 1.0;
        }
        theme::ease_out_cubic(((self.t - self.since) / 0.32) as f32)
    }

    /// True while the enter animation runs.
    pub fn entering(&self) -> bool {
        !self.reduce_motion && self.t - self.since < 0.32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for (s, _) in Screen::NAV {
            assert_eq!(Screen::parse(s.id()), Some(s));
        }
        assert_eq!(Screen::parse("nope"), None);
    }
}
