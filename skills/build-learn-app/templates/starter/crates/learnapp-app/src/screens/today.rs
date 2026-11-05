//! Today: the plan for one short session, a typographic list, no cards.

use super::{Action, Frame};
use crate::theme::{self, ACCENT, INK, INK_2, INK_3, LATE, LINE, LINE_STRONG, ty};
use crate::widgets;
use egui::{Align2, Pos2, Rect, Stroke, Ui, Vec2};

/// One row of the plan.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanRow {
    /// Kind in caps ("WARM-UP", "REVIEW", "NEW").
    pub kind: &'static str,
    /// Title of the item.
    pub title: String,
    /// Planned minutes.
    pub minutes: u32,
}

/// What Today shows.
#[derive(Clone, Debug, PartialEq)]
pub struct TodayView {
    /// "Thursday 5 November".
    pub date: String,
    /// A session was completed today.
    pub done_today: bool,
    /// The plan.
    pub plan: Vec<PlanRow>,
    /// Timing spread of the last run (ms).
    pub spread_ms: Option<f64>,
    /// Practice days of the last seven days, oldest first.
    pub ring: [bool; 7],
    /// Practice days this week and the weekly target.
    pub week: (usize, usize),
    /// A permanent problem note (saving is off).
    pub note: Option<String>,
}

/// Paints Today.
pub fn show(ui: &mut Ui, content: Rect, f: &Frame, v: &TodayView) -> Option<Action> {
    let mut action = None;
    let e = f.enter();
    ui.set_opacity(e);
    let y0 = content.top() + (1.0 - e) * 10.0;
    let left = content.left();
    let col = content.width() * 0.58;

    widgets::caps_at(ui, Pos2::new(left, y0), Align2::LEFT_TOP, &v.date, INK_3);
    let (title, cue) = if v.done_today {
        (
            "Done for today.",
            "Spacing practice across days is what makes it stick. Come back tomorrow, or practise more now.",
        )
    } else {
        (
            "Today",
            "One short session: a warm-up, what is due, then one new thing.",
        )
    };
    let t = widgets::text_at(
        ui,
        Pos2::new(left - 2.0, y0 + 22.0),
        Align2::LEFT_TOP,
        title,
        ty::hero(),
        INK,
    );
    let c = widgets::paragraph_at(
        ui,
        Pos2::new(left, t.bottom() + 12.0),
        cue,
        ty::body(),
        INK_2,
        col - 40.0,
    );

    // The plan: numbered rows separated by hairlines.
    let mut y = c.bottom() + theme::S7;
    let row_h = 58.0;
    let p = ui.painter().clone();
    p.line_segment(
        [Pos2::new(left, y), Pos2::new(left + col, y)],
        Stroke::new(1.0, LINE),
    );
    for (i, row) in v.plan.iter().enumerate() {
        let mid = y + row_h / 2.0;
        widgets::text_at(
            ui,
            Pos2::new(left, mid),
            Align2::LEFT_CENTER,
            &format!("{:02}", i + 1),
            ty::mono(),
            INK_3,
        );
        widgets::caps_at(
            ui,
            Pos2::new(left + 44.0, mid),
            Align2::LEFT_CENTER,
            row.kind,
            INK_3,
        );
        widgets::text_at(
            ui,
            Pos2::new(left + 150.0, mid),
            Align2::LEFT_CENTER,
            &row.title,
            ty::heading(),
            INK,
        );
        widgets::caps_at(
            ui,
            Pos2::new(left + col, mid),
            Align2::RIGHT_CENTER,
            &format!("{} min", row.minutes),
            INK_3,
        );
        y += row_h;
        p.line_segment(
            [Pos2::new(left, y), Pos2::new(left + col, y)],
            Stroke::new(1.0, LINE),
        );
    }

    let label = if v.done_today {
        "Practise more"
    } else {
        "Start session"
    };
    let mut row = widgets::row_ui(
        ui,
        Rect::from_min_size(Pos2::new(left, y + theme::S6), Vec2::new(col, 44.0)),
    );
    if widgets::primary_button(&mut row, label, Some("SPACE")).clicked() {
        action = Some(Action::Start);
    }

    // Right column: measured numbers only.
    let rx = left + col + theme::S8;
    let mut ry = y0 + 30.0;
    widgets::caps_at(
        ui,
        Pos2::new(rx, ry),
        Align2::LEFT_TOP,
        "Timing spread",
        INK_3,
    );
    ry += 22.0;
    let value = v
        .spread_ms
        .map(|s| format!("{s:.0}"))
        .unwrap_or_else(|| "—".into());
    let r = widgets::readout(
        ui,
        Pos2::new(rx, ry),
        &value,
        64.0,
        if v.spread_ms.is_some() { "ms" } else { "" },
    );
    ry = r.bottom() + 6.0;
    let hint = if v.spread_ms.is_some() {
        "Lower is steadier."
    } else {
        "Your first run measures it."
    };
    widgets::text_at(
        ui,
        Pos2::new(rx, ry),
        Align2::LEFT_TOP,
        hint,
        ty::small(),
        INK_3,
    );

    ry += theme::S8;
    widgets::caps_at(ui, Pos2::new(rx, ry), Align2::LEFT_TOP, "This week", INK_3);
    ry += 30.0;
    for (i, done) in v.ring.iter().enumerate() {
        let c = Pos2::new(rx + 7.0 + i as f32 * 26.0, ry + 7.0);
        if *done {
            p.circle_filled(c, 7.0, ACCENT);
        } else {
            p.circle_stroke(c, 6.5, Stroke::new(1.0, LINE_STRONG));
        }
    }
    ry += 30.0;
    let w = widgets::text_at(
        ui,
        Pos2::new(rx, ry),
        Align2::LEFT_TOP,
        &format!("{} of {} days", v.week.0, v.week.1),
        ty::heading(),
        INK,
    );
    widgets::text_at(
        ui,
        Pos2::new(rx, w.bottom() + 6.0),
        Align2::LEFT_TOP,
        "A rest day never breaks anything.",
        ty::small(),
        INK_3,
    );

    if let Some(note) = &v.note {
        let ny = content.bottom() - 20.0;
        let k = widgets::caps_at(
            ui,
            Pos2::new(left, ny),
            Align2::LEFT_CENTER,
            "Not saving",
            LATE,
        );
        widgets::text_at(
            ui,
            Pos2::new(k.right() + 12.0, ny),
            Align2::LEFT_CENTER,
            note,
            ty::small(),
            INK_2,
        );
    }
    action
}
