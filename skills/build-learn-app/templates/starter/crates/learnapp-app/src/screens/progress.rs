//! Progress: measured skill over time. No points, no XP, no streak loss.

use super::{Action, Frame};
use crate::theme::{ACCENT, INK, INK_2, INK_3, LINE, ty};
use crate::widgets;
use egui::{Align2, Pos2, Rect, Stroke, Ui};

/// One past run.
#[derive(Clone, Debug, PartialEq)]
pub struct RunRow {
    /// "5 Nov".
    pub day: String,
    /// Accuracy 0..1.
    pub accuracy: f64,
    /// Spread in ms.
    pub spread_ms: f64,
    /// The gate was cleared.
    pub passed: bool,
}

/// What Progress shows.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressView {
    /// Runs, oldest first.
    pub runs: Vec<RunRow>,
    /// Practice days in total.
    pub days: usize,
}

/// Paints Progress.
pub fn show(ui: &mut Ui, content: Rect, f: &Frame, v: &ProgressView) -> Option<Action> {
    let e = f.enter();
    ui.set_opacity(e);
    let left = content.left();
    let y0 = content.top() + (1.0 - e) * 10.0;
    widgets::caps_at(ui, Pos2::new(left, y0), Align2::LEFT_TOP, "Progress", INK_3);
    let t = widgets::text_at(
        ui,
        Pos2::new(left - 2.0, y0 + 22.0),
        Align2::LEFT_TOP,
        "How steady you are",
        ty::display(),
        INK,
    );
    if v.runs.is_empty() {
        widgets::text_at(
            ui,
            Pos2::new(left, t.bottom() + 14.0),
            Align2::LEFT_TOP,
            "No runs yet. Your first run shows up here.",
            ty::body(),
            INK_2,
        );
        return None;
    }
    let mut x = left;
    let y = t.bottom() + 40.0;
    let best = v
        .runs
        .iter()
        .map(|r| r.spread_ms)
        .fold(f64::INFINITY, f64::min);
    for (label, value, unit) in [
        ("Runs", v.runs.len().to_string(), ""),
        ("Practice days", v.days.to_string(), ""),
        ("Best spread", format!("{best:.0}"), "ms"),
    ] {
        widgets::caps_at(ui, Pos2::new(x, y), Align2::LEFT_TOP, label, INK_3);
        let r = widgets::readout(ui, Pos2::new(x, y + 20.0), &value, 52.0, unit);
        x = r.right().max(x + 170.0) + 48.0;
    }

    // Spread per run: a thin line, the last point in the accent.
    let chart = Rect::from_min_size(
        Pos2::new(left, y + 120.0),
        egui::vec2(content.width() * 0.55, 120.0),
    );
    let p = ui.painter();
    p.line_segment(
        [chart.left_bottom(), chart.right_bottom()],
        Stroke::new(1.0, LINE),
    );
    let max = v.runs.iter().map(|r| r.spread_ms).fold(1.0, f64::max);
    let n = v.runs.len();
    let pts: Vec<Pos2> = v
        .runs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let tx = if n > 1 {
                i as f32 / (n - 1) as f32
            } else {
                1.0
            };
            Pos2::new(
                chart.left() + tx * chart.width(),
                chart.bottom() - (r.spread_ms / max) as f32 * chart.height(),
            )
        })
        .collect();
    if pts.len() > 1 {
        p.add(egui::Shape::line(pts.clone(), Stroke::new(1.5, INK_2)));
    }
    if let Some(last) = pts.last() {
        p.circle_filled(*last, 4.0, ACCENT);
    }
    widgets::caps_at(
        ui,
        Pos2::new(chart.left(), chart.bottom() + 10.0),
        Align2::LEFT_TOP,
        "Spread per run (lower is steadier)",
        INK_3,
    );

    // The last runs as a typographic list.
    let lx = chart.right() + 64.0;
    let mut ly = chart.top();
    for r in v.runs.iter().rev().take(8) {
        widgets::text_at(
            ui,
            Pos2::new(lx, ly),
            Align2::LEFT_TOP,
            &r.day,
            ty::mono(),
            INK_3,
        );
        widgets::text_at(
            ui,
            Pos2::new(lx + 70.0, ly),
            Align2::LEFT_TOP,
            &format!("{:.0} %  ·  {:.0} ms", r.accuracy * 100.0, r.spread_ms),
            ty::mono(),
            INK,
        );
        if r.passed {
            widgets::caps_at(
                ui,
                Pos2::new(lx + 230.0, ly + 1.0),
                Align2::LEFT_TOP,
                "passed",
                INK_2,
            );
        }
        ly += 24.0;
    }
    None
}
