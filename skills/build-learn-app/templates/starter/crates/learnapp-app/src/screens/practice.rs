//! Practice: one scored run on a lane of targets. The subject itself is the
//! main widget (here a timing lane and two keys); replace it with the real
//! exercise surface of the app. Sound is the main time cue in a real app:
//! visuals move smoothly and never flash on the beat.

use super::{Action, Frame};
use crate::theme::{self, ACCENT, INK, INK_2, INK_3, INK_4, LINE, LINE_STRONG, ty};
use crate::widgets::{self, ChipState};
use egui::{Align2, CornerRadius, Pos2, Rect, Stroke, Ui, Vec2};
use learnapp_core::Summary;

/// Where the run is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// Nothing runs yet.
    #[default]
    Ready,
    /// Counting in before the first target.
    CountIn,
    /// Targets pass the playhead.
    Playing,
    /// The run is scored.
    Summary,
}

impl Phase {
    /// Stable id for `state.json`.
    pub fn id(self) -> &'static str {
        match self {
            Phase::Ready => "ready",
            Phase::CountIn => "count_in",
            Phase::Playing => "playing",
            Phase::Summary => "summary",
        }
    }
}

/// One lane row: a key and its label.
#[derive(Clone, Debug, PartialEq)]
pub struct LaneKey {
    /// macOS key code.
    pub code: u16,
    /// Label printed on the key ("F").
    pub label: String,
    /// 0..1 glow after a press.
    pub glow: f32,
}

/// What Practice shows.
#[derive(Clone, Debug, PartialEq)]
pub struct PracticeView {
    /// Meta line in caps.
    pub meta: String,
    /// Item title.
    pub title: String,
    /// One external-focus cue (names the result, not the fingers).
    pub cue: String,
    /// Run phase.
    pub phase: Phase,
    /// Tempo in BPM.
    pub bpm: f64,
    /// Beat length in ms.
    pub beat_ms: f64,
    /// Time now relative to the first target (ms; negative while counting in).
    pub now_ms: f64,
    /// Targets: (ms, row index).
    pub targets: Vec<(f64, usize)>,
    /// Hits: (ms, row, offset to its target or None for an extra).
    pub hits: Vec<(f64, usize, Option<f64>)>,
    /// Good window (ms): only hits outside it are coloured.
    pub good_ms: f64,
    /// Lane rows.
    pub keys: Vec<LaneKey>,
    /// The scored result.
    pub summary: Option<Summary>,
    /// One honest line about the result.
    pub verdict: Option<String>,
}

/// Paints Practice.
pub fn show(ui: &mut Ui, content: Rect, f: &Frame, v: &PracticeView) -> Option<Action> {
    let mut action = None;
    let e = f.enter();
    ui.set_opacity(e);
    let left = content.left();
    let y0 = content.top() + (1.0 - e) * 10.0;
    widgets::caps_at(ui, Pos2::new(left, y0), Align2::LEFT_TOP, &v.meta, INK_3);
    let t = widgets::text_at(
        ui,
        Pos2::new(left - 2.0, y0 + 22.0),
        Align2::LEFT_TOP,
        &v.title,
        ty::display(),
        INK,
    );
    widgets::text_at(
        ui,
        Pos2::new(left, t.bottom() + 10.0),
        Align2::LEFT_TOP,
        &v.cue,
        ty::body(),
        INK_2,
    );

    // The lane: rows = keys, x = time. While a run plays, the playhead is
    // fixed and targets move left. After the run the whole take is shown at
    // once, so every hit, miss (an empty target) and extra stays visible.
    let lane = Rect::from_min_size(
        Pos2::new(left, t.bottom() + 70.0),
        Vec2::new(content.width(), 64.0 * v.keys.len() as f32),
    );
    let p = ui.painter().clone();
    let whole = v.phase == Phase::Summary && !v.targets.is_empty();
    let first_ms = v.targets.first().map(|t| t.0).unwrap_or(0.0) - v.beat_ms;
    let last_ms = v.targets.last().map(|t| t.0).unwrap_or(0.0) + v.beat_ms;
    let head_x = lane.left() + 64.0 + (lane.width() - 64.0) * 0.28;
    let px_per_ms = ((lane.width() - 64.0) / (v.beat_ms * 6.0) as f32).max(0.01);
    let span = lane.width() - 64.0 - 24.0;
    let x_of = |ms: f64| {
        if whole {
            lane.left() + 76.0 + ((ms - first_ms) / (last_ms - first_ms)) as f32 * span
        } else {
            head_x + ((ms - v.now_ms) as f32) * px_per_ms
        }
    };
    let row_y = |r: usize| lane.top() + 32.0 + r as f32 * 64.0;
    let clip = Rect::from_min_max(Pos2::new(lane.left() + 64.0, lane.top()), lane.max);
    let lp = p.with_clip_rect(clip);
    // Beat lines: the beats around the playhead, or every beat of the take.
    let (first_beat, beats) = if whole {
        (
            (first_ms / v.beat_ms).floor() as i64,
            ((last_ms - first_ms) / v.beat_ms).ceil() as i64 + 1,
        )
    } else {
        (
            ((v.now_ms - v.beat_ms * 3.0) / v.beat_ms).floor() as i64,
            10,
        )
    };
    for b in first_beat..first_beat + beats {
        let x = x_of(b as f64 * v.beat_ms);
        lp.line_segment(
            [Pos2::new(x, lane.top()), Pos2::new(x, lane.bottom())],
            Stroke::new(1.0, LINE),
        );
    }
    for (r, k) in v.keys.iter().enumerate() {
        let y = row_y(r);
        let chip = Rect::from_center_size(Pos2::new(lane.left() + 20.0, y), Vec2::splat(36.0));
        widgets::key_chip(
            ui,
            chip,
            &k.label,
            ChipState {
                active: true,
                pressed: k.glow,
                countdown: None,
            },
        );
        lp.line_segment(
            [Pos2::new(clip.left(), y), Pos2::new(clip.right(), y)],
            Stroke::new(1.0, LINE),
        );
    }
    for &(ms, r) in &v.targets {
        let x = x_of(ms);
        let rect = Rect::from_center_size(Pos2::new(x, row_y(r)), Vec2::new(10.0, 26.0));
        let passed = !whole && ms < v.now_ms - 120.0;
        let colour = if passed { INK_4 } else { INK_3 };
        lp.rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, colour),
            egui::StrokeKind::Inside,
        );
    }
    for &(ms, r, offset) in &v.hits {
        let x = x_of(ms);
        let c = match offset {
            Some(o) if o.abs() <= v.good_ms => INK,
            Some(o) => theme::offset_color(o, v.good_ms),
            None => ACCENT,
        };
        lp.circle_filled(Pos2::new(x, row_y(r)), 5.0, c);
    }
    if !whole {
        lp.line_segment(
            [
                Pos2::new(head_x, lane.top() - 8.0),
                Pos2::new(head_x, lane.bottom() + 8.0),
            ],
            Stroke::new(1.5, ACCENT),
        );
    }
    p.line_segment(
        [
            Pos2::new(lane.left(), lane.bottom() + 20.0),
            Pos2::new(lane.right(), lane.bottom() + 20.0),
        ],
        Stroke::new(1.0, LINE_STRONG),
    );

    if v.phase == Phase::CountIn {
        let left_beats = (-v.now_ms / v.beat_ms).ceil().max(1.0);
        widgets::text_at(
            ui,
            Pos2::new(lane.right(), lane.top() - 24.0),
            Align2::RIGHT_BOTTOM,
            &format!("{left_beats:.0}"),
            ty::readout(96.0),
            INK,
        );
    }

    // Transport row and result.
    let ty0 = lane.bottom() + 52.0;
    let r = widgets::readout(
        ui,
        Pos2::new(left, ty0),
        &format!("{:.0}", v.bpm),
        44.0,
        "bpm",
    );
    let mut row = widgets::row_ui(
        ui,
        Rect::from_min_size(
            Pos2::new(r.right() + theme::S7, ty0 + 2.0),
            Vec2::new(520.0, 44.0),
        ),
    );
    match v.phase {
        Phase::Ready => {
            if widgets::primary_button(&mut row, "Start", Some("SPACE")).clicked() {
                action = Some(Action::Start);
            }
        }
        Phase::CountIn | Phase::Playing => {
            if widgets::text_button(&mut row, "Leave session").clicked() {
                action = Some(Action::Leave);
            }
        }
        Phase::Summary => {
            if widgets::primary_button(&mut row, "Again", Some("SPACE")).clicked() {
                action = Some(Action::Start);
            }
            row.add_space(theme::S5);
            if widgets::text_button(&mut row, "Back to Today").clicked() {
                action = Some(Action::Go(super::Screen::Today));
            }
        }
    }
    if let (Some(s), Some(verdict)) = (&v.summary, &v.verdict) {
        let sy = ty0 + 96.0;
        let mut x = left;
        for (label, value, unit) in [
            ("Accuracy", format!("{:.0}", s.accuracy * 100.0), "%"),
            ("Bias", format!("{:+.0}", s.bias_ms), "ms"),
            ("Spread", format!("{:.0}", s.spread_ms), "ms"),
        ] {
            widgets::caps_at(ui, Pos2::new(x, sy), Align2::LEFT_TOP, label, INK_3);
            let r = widgets::readout(ui, Pos2::new(x, sy + 20.0), &value, 52.0, unit);
            x = r.right().max(x + 150.0) + theme::S7;
        }
        widgets::text_at(
            ui,
            Pos2::new(left, sy + 96.0),
            Align2::LEFT_TOP,
            verdict,
            ty::heading(),
            INK,
        );
    }

    // The keys under the hands, lit on press.
    let ky = content.bottom() - 58.0;
    for (i, k) in v.keys.iter().enumerate() {
        let rect = Rect::from_min_size(Pos2::new(left + i as f32 * 62.0, ky), Vec2::splat(54.0));
        widgets::keycap(
            ui,
            rect,
            &k.label,
            None,
            None,
            ChipState {
                active: true,
                pressed: k.glow,
                countdown: None,
            },
        );
    }
    action
}
