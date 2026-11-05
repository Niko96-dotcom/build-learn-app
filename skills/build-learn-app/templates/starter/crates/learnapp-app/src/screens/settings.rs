//! Settings: few, plain, reversible.

use super::{Action, Frame};
use crate::theme::{self, ACCENT, INK, INK_2, INK_3, ty};
use crate::widgets;
use egui::{Align2, Pos2, Rect, Ui, Vec2};

/// What Settings shows.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsView {
    /// Reduce motion is on.
    pub reduce_motion: bool,
    /// Output level 0..1.
    pub level: f32,
    /// Data folder, for display.
    pub data_dir: String,
    /// The first click of "Reset progress" happened.
    pub reset_asked: bool,
}

/// Paints Settings.
pub fn show(ui: &mut Ui, content: Rect, f: &Frame, v: &SettingsView) -> Option<Action> {
    let mut action = None;
    let e = f.enter();
    ui.set_opacity(e);
    let left = content.left();
    let y0 = content.top() + (1.0 - e) * 10.0;
    widgets::caps_at(
        ui,
        Pos2::new(left, y0),
        Align2::LEFT_TOP,
        "Local only · no account",
        INK_3,
    );
    let t = widgets::text_at(
        ui,
        Pos2::new(left - 2.0, y0 + 22.0),
        Align2::LEFT_TOP,
        "Settings",
        ty::display(),
        INK,
    );
    let mut y = t.bottom() + theme::S7;
    let w = 560.0;

    let mut row = widgets::row_ui(
        ui,
        Rect::from_min_size(Pos2::new(left, y), Vec2::new(w, 32.0)),
    );
    let mut rm = v.reduce_motion;
    if widgets::switch(&mut row, "Reduce motion", &mut rm).changed() {
        action = Some(Action::ReduceMotion(rm));
    }
    y += 56.0;
    widgets::caps_at(ui, Pos2::new(left, y), Align2::LEFT_TOP, "Level", INK_3);
    let mut row = widgets::row_ui(
        ui,
        Rect::from_min_size(Pos2::new(left, y + 18.0), Vec2::new(w, 32.0)),
    );
    let mut level = v.level;
    if widgets::slider(&mut row, "Level", &mut level, 0.0..=1.0, 320.0) {
        action = Some(Action::Level(level));
    }
    widgets::text_at(
        ui,
        Pos2::new(left + 340.0, y + 34.0),
        Align2::LEFT_CENTER,
        &format!("{:.0} %", v.level * 100.0),
        ty::mono(),
        INK_2,
    );
    y += 92.0;
    widgets::caps_at(
        ui,
        Pos2::new(left, y),
        Align2::LEFT_TOP,
        "Data folder",
        INK_3,
    );
    widgets::text_at(
        ui,
        Pos2::new(left, y + 20.0),
        Align2::LEFT_TOP,
        &v.data_dir,
        ty::mono(),
        INK_2,
    );
    y += 76.0;
    let mut row = widgets::row_ui(
        ui,
        Rect::from_min_size(Pos2::new(left, y), Vec2::new(w * 1.6, 32.0)),
    );
    if v.reset_asked {
        row.label(
            egui::RichText::new("This moves your profile and history to the Trash.")
                .font(ty::body())
                .color(INK_2),
        );
        row.add_space(theme::S5);
        if widgets::text_button_colored(&mut row, "Move to Trash", ACCENT).clicked() {
            action = Some(Action::ResetConfirm);
        }
        row.add_space(theme::S4);
        if widgets::text_button(&mut row, "Keep it").clicked() {
            action = Some(Action::ResetCancel);
        }
    } else if widgets::text_button(&mut row, "Reset progress").clicked() {
        action = Some(Action::ResetAsk);
    }
    action
}
