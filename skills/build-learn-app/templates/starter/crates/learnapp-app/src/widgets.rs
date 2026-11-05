//! Hand-drawn controls. Every interactive widget reports
//! an accessibility label equal to its visible text, so the verification
//! skill can find it through the macOS accessibility tree.

use crate::theme::{self, ACCENT, BG, INK, INK_2, INK_3, INK_4, LINE, LINE_STRONG, ty};
use egui::text::{LayoutJob, TextFormat};
use egui::{
    Align2, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Response, Role, Sense, Stroke,
    StrokeKind, Ui, Vec2, WidgetInfo,
};
use std::sync::Arc;

/// Lays out one line of text.
pub fn galley(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    letter_spacing: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font,
            color,
            extra_letter_spacing: letter_spacing,
            ..Default::default()
        },
    );
    ui.fonts_mut(|f| f.layout_job(job))
}

/// Lays out wrapped text.
pub fn galley_wrapped(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.wrap.max_width = width;
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    ui.fonts_mut(|f| f.layout_job(job))
}

/// Paints text at `pos` with an anchor; returns the rect.
pub fn text_at(
    ui: &Ui,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    font: FontId,
    color: Color32,
) -> Rect {
    let g = galley(ui, text, font, color, 0.0);
    let rect = anchor.anchor_size(pos, g.size());
    ui.painter().galley(rect.min, g, color);
    rect
}

/// Paints wrapped text with its top-left at `pos`; returns the rect.
pub fn paragraph_at(
    ui: &Ui,
    pos: Pos2,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
) -> Rect {
    let g = galley_wrapped(ui, text, font, color, width);
    let rect = Rect::from_min_size(pos, g.size());
    ui.painter().galley(pos, g, color);
    rect
}

/// Uppercase mono label with tracking.
pub fn caps_at(ui: &Ui, pos: Pos2, anchor: Align2, text: &str, color: Color32) -> Rect {
    let g = galley(ui, &text.to_uppercase(), ty::label(), color, 1.2);
    let rect = anchor.anchor_size(pos, g.size());
    ui.painter().galley(rect.min, g, color);
    rect
}

/// Allocates and paints an uppercase label in the layout.
pub fn caps(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let g = galley(ui, &text.to_uppercase(), ty::label(), color, 1.2);
    let (rect, resp) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    resp
}

/// Allocates and paints text in the layout.
pub fn text(ui: &mut Ui, text: &str, font: FontId, color: Color32) -> Response {
    let g = galley(ui, text, font, color, 0.0);
    let (rect, resp) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    resp
}

fn press_anim(ui: &Ui, resp: &Response) -> (f32, f32) {
    let hover = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.12);
    let press =
        ui.ctx()
            .animate_bool_with_time(resp.id.with("p"), resp.is_pointer_button_down_on(), 0.06);
    (hover, press)
}

fn pointer_on_hover(ui: &Ui, resp: &Response) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

/// The one primary action of a screen: a solid accent pill with a key hint.
pub fn primary_button(ui: &mut Ui, label: &str, hint: Option<&str>) -> Response {
    let font = theme::font(theme::SANS_MEDIUM, 15.0);
    let g = galley(ui, label, font, BG, 0.0);
    let hint_g = hint.map(|h| galley(ui, h, ty::label(), theme::alpha(BG, 0.55), 1.0));
    let extra = hint_g.as_ref().map(|h| h.size().x + 14.0).unwrap_or(0.0);
    let size = Vec2::new(g.size().x + 44.0 + extra, 44.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    if ui.is_rect_visible(rect) {
        let (hover, press) = press_anim(ui, &resp);
        let r = rect.shrink(press * 1.5);
        let fill = if ui.is_enabled() {
            theme::mix(ACCENT, theme::ACCENT_HOVER, hover)
        } else {
            INK_4
        };
        ui.painter().rect_filled(r, CornerRadius::same(22), fill);
        let text_pos = Pos2::new(r.left() + 22.0, r.center().y - g.size().y / 2.0);
        ui.painter().galley(text_pos, g.clone(), BG);
        if let Some(h) = hint_g {
            let p = Pos2::new(
                text_pos.x + g.size().x + 14.0,
                r.center().y - h.size().y / 2.0,
            );
            ui.painter().galley(p, h, BG);
        }
    }
    pointer_on_hover(ui, &resp);
    resp
}

/// A quiet text action with an underline that grows on hover.
pub fn text_button(ui: &mut Ui, label: &str) -> Response {
    text_button_colored(ui, label, INK_2)
}

/// [`text_button`] in another colour (accent for "Measure again").
pub fn text_button_colored(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let font = theme::font(theme::SANS_MEDIUM, 14.0);
    let g = galley(ui, label, font.clone(), color, 0.0);
    let size = g.size() + Vec2::new(0.0, 8.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    if ui.is_rect_visible(rect) {
        let (hover, _) = press_anim(ui, &resp);
        let c = theme::mix(color, INK, hover);
        ui.painter()
            .galley(rect.min, galley(ui, label, font, c, 0.0), c);
        let y = rect.bottom() - 2.0;
        let w = rect.width() * hover;
        if w > 0.5 {
            ui.painter().line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.left() + w, y)],
                Stroke::new(1.0, c),
            );
        }
    }
    pointer_on_hover(ui, &resp);
    resp
}

/// An outlined pill for secondary actions.
pub fn outline_button(ui: &mut Ui, label: &str) -> Response {
    let font = theme::font(theme::SANS_MEDIUM, 14.0);
    let g = galley(ui, label, font, INK, 0.0);
    let size = Vec2::new(g.size().x + 32.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    if ui.is_rect_visible(rect) {
        let (hover, press) = press_anim(ui, &resp);
        let r = rect.shrink(press);
        let stroke = theme::mix(LINE_STRONG, INK_2, hover);
        ui.painter().rect_stroke(
            r,
            CornerRadius::same(18),
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
        let c = if ui.is_enabled() { INK } else { INK_3 };
        ui.painter().galley(
            Pos2::new(r.left() + 16.0, r.center().y - g.size().y / 2.0),
            g,
            c,
        );
    }
    pointer_on_hover(ui, &resp);
    resp
}

/// A selectable word: dim when off, bright with a small dot when on.
pub fn choice(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let font = theme::font(theme::SANS_MEDIUM, 15.0);
    let g = galley(ui, label, font.clone(), INK, 0.0);
    let size = g.size() + Vec2::new(18.0, 10.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::selected(Role::CheckBox, ui.is_enabled(), selected, label));
    if ui.is_rect_visible(rect) {
        let (hover, _) = press_anim(ui, &resp);
        let on = ui
            .ctx()
            .animate_bool_with_time(resp.id.with("on"), selected, 0.16);
        let c = theme::mix(theme::mix(INK_3, INK_2, hover), INK, on);
        ui.painter().galley(
            Pos2::new(rect.left() + 18.0, rect.top() + 5.0),
            galley(ui, label, font, c, 0.0),
            c,
        );
        let centre = Pos2::new(rect.left() + 6.0, rect.center().y);
        ui.painter().circle_stroke(
            centre,
            3.5,
            Stroke::new(1.0, theme::mix(INK_4, INK_3, hover)),
        );
        if on > 0.01 {
            ui.painter().circle_filled(
                centre,
                3.5 * theme::ease_out_back(on),
                theme::alpha(ACCENT, on),
            );
        }
    }
    pointer_on_hover(ui, &resp);
    resp
}

/// A row of options with an animated accent underline under the selected one.
/// Returns the index of a clicked option.
pub fn segmented(ui: &mut Ui, id: &str, options: &[&str], selected: usize) -> Option<usize> {
    let mut clicked = None;
    let font = theme::font(theme::SANS_MEDIUM, 14.0);
    let mut rects = Vec::new();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        for (i, opt) in options.iter().enumerate() {
            let g = galley(ui, opt, font.clone(), INK, 0.0);
            let (rect, resp) =
                ui.allocate_exact_size(g.size() + Vec2::new(0.0, 10.0), Sense::click());
            resp.widget_info(|| WidgetInfo::selected(Role::Tab, true, i == selected, *opt));
            let (hover, _) = press_anim(ui, &resp);
            let c = if i == selected {
                INK
            } else {
                theme::mix(INK_3, INK_2, hover)
            };
            ui.painter()
                .galley(rect.min, galley(ui, opt, font.clone(), c, 0.0), c);
            if resp.clicked() {
                clicked = Some(i);
            }
            pointer_on_hover(ui, &resp);
            rects.push(rect);
        }
    });
    if let Some(target) = rects.get(selected) {
        let ctx = ui.ctx();
        let base = egui::Id::unique(("seg", id));
        let x0 = ctx.animate_value_with_time(base.with("x0"), target.left(), 0.22);
        let x1 = ctx.animate_value_with_time(base.with("x1"), target.right(), 0.22);
        let y = target.bottom() - 1.0;
        ui.painter().line_segment(
            [Pos2::new(x0, y), Pos2::new(x1, y)],
            Stroke::new(1.5, ACCENT),
        );
    }
    clicked
}

/// A thin horizontal slider. Returns true when the value changed.
pub fn slider(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    width: f32,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 28.0), Sense::click_and_drag());
    let (lo, hi) = (*range.start(), *range.end());
    resp.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Slider, true, label);
        info.value = Some(*value as f64);
        info
    });
    let mut changed = false;
    if let Some(p) = resp.interact_pointer_pos() {
        let t = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let v = lo + t * (hi - lo);
        if (v - *value).abs() > f32::EPSILON {
            *value = v;
            changed = true;
        }
    }
    // Accessibility (VoiceOver, the verification driver): steps of 5 % and
    // set-value, as egui's own slider.
    let step = (hi - lo) / 20.0;
    let (steps, set) = ui.input(|i| {
        use egui::accesskit::{Action, ActionData};
        let steps = i.num_accesskit_action_requests(resp.id, Action::Increment) as f32
            - i.num_accesskit_action_requests(resp.id, Action::Decrement) as f32;
        let set = i
            .accesskit_action_requests(resp.id, Action::SetValue)
            .filter_map(|r| match r.data {
                Some(ActionData::NumericValue(v)) => Some(v as f32),
                _ => None,
            })
            .last();
        (steps, set)
    });
    if steps != 0.0 || set.is_some() {
        let v = set.unwrap_or(*value + steps * step).clamp(lo, hi);
        if (v - *value).abs() > f32::EPSILON {
            *value = v;
            changed = true;
        }
    }
    ui.ctx().accesskit_node_builder(resp.id, |b| {
        use egui::accesskit::Action;
        b.set_min_numeric_value(f64::from(lo));
        b.set_max_numeric_value(f64::from(hi));
        b.set_numeric_value_step(f64::from(step));
        b.add_action(Action::SetValue);
        b.add_action(Action::Increment);
        b.add_action(Action::Decrement);
    });
    let t = ((*value - lo) / (hi - lo)).clamp(0.0, 1.0);
    let y = rect.center().y;
    let x = rect.left() + t * rect.width();
    let p = ui.painter();
    p.line_segment(
        [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
        Stroke::new(1.0, LINE_STRONG),
    );
    p.line_segment(
        [Pos2::new(rect.left(), y), Pos2::new(x, y)],
        Stroke::new(1.5, INK_2),
    );
    let hover =
        ui.ctx()
            .animate_bool_with_time(resp.id.with("h"), resp.hovered() || resp.dragged(), 0.1);
    p.circle_filled(Pos2::new(x, y), 5.0 + 2.0 * hover, INK);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    changed
}

/// A switch drawn as a short track with a dot.
pub fn switch(ui: &mut Ui, label: &str, on: &mut bool) -> Response {
    let font = theme::font(theme::SANS_MEDIUM, 14.0);
    let g = galley(ui, label, font, INK_2, 0.0);
    let size = Vec2::new(34.0 + 12.0 + g.size().x, 22.0f32.max(g.size().y));
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    resp.widget_info(|| WidgetInfo::selected(Role::Switch, true, *on, label));
    let t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("on"), *on, 0.16);
    let track = Rect::from_min_size(
        Pos2::new(rect.left(), rect.center().y - 9.0),
        Vec2::new(34.0, 18.0),
    );
    let p = ui.painter();
    p.rect_stroke(
        track,
        CornerRadius::same(9),
        Stroke::new(1.0, theme::mix(LINE_STRONG, ACCENT, t)),
        StrokeKind::Inside,
    );
    let x = track.left() + 9.0 + t * 16.0;
    p.circle_filled(
        Pos2::new(x, track.center().y),
        5.0,
        theme::mix(INK_3, ACCENT, t),
    );
    p.galley(
        Pos2::new(track.right() + 12.0, rect.center().y - g.size().y / 2.0),
        g,
        INK_2,
    );
    pointer_on_hover(ui, &resp);
    resp
}

/// A full-width row that reacts to hover with a soft wash, no border.
/// `label` is the row's visible title (its accessibility label).
pub fn row(ui: &mut Ui, height: f32, label: &str) -> (Rect, Response) {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let hover = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.12);
    if hover > 0.0 {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(6),
            theme::alpha(theme::WASH, hover),
        );
    }
    pointer_on_hover(ui, &resp);
    (rect, resp)
}

/// A hairline across the available width.
pub fn rule(ui: &mut Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 1.0), Sense::hover());
    ui.painter().line_segment(
        [rect.left_center(), rect.right_center()],
        Stroke::new(1.0, LINE),
    );
}

/// Combo boxes without a box: text and chevron, a hairline on hover.
pub fn quiet_combos(ui: &mut Ui) {
    let w = &mut ui.visuals_mut().widgets;
    w.inactive.bg_stroke = Stroke::new(1.0, theme::alpha(LINE, 0.0));
    w.inactive.weak_bg_fill = Color32::TRANSPARENT;
    w.inactive.bg_fill = Color32::TRANSPARENT;
    w.hovered.weak_bg_fill = theme::WASH;
    w.hovered.bg_stroke = Stroke::new(1.0, LINE_STRONG);
    w.open.weak_bg_fill = theme::WASH;
    w.open.bg_stroke = Stroke::new(1.0, INK_3);
    w.inactive.corner_radius = CornerRadius::same(6);
    w.hovered.corner_radius = CornerRadius::same(6);
    w.open.corner_radius = CornerRadius::same(6);
}

/// A child `Ui` in `rect` with a horizontal layout (action rows).
pub fn row_ui(ui: &mut Ui, rect: Rect) -> Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    )
}

/// A child `Ui` in `rect` laid out right to left (actions on the right edge).
pub fn row_ui_right(ui: &mut Ui, rect: Rect) -> Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    )
}

/// A big dot-matrix numeral with a small caps unit after it; returns the union rect.
pub fn readout(ui: &Ui, pos: Pos2, value: &str, size: f32, unit: &str) -> Rect {
    let r = text_at(ui, pos, Align2::LEFT_TOP, value, ty::readout(size), INK);
    if unit.is_empty() {
        return r;
    }
    let u = caps_at(
        ui,
        Pos2::new(r.right() + 8.0, r.bottom() - size * 0.12),
        Align2::LEFT_BOTTOM,
        unit,
        INK_3,
    );
    r.union(u)
}

// Key chips -------------------------------------------------------------------

/// Visual state of a key chip: the keyboard widget's caps and the roll's row labels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChipState {
    /// The key belongs to the current material (labels bright).
    pub active: bool,
    /// Held down, 0..1 (animated: 1 while held, eases out on release).
    pub pressed: f32,
    /// Countdown fill 0..1 in the beat before the key's next target note.
    pub countdown: Option<f32>,
}

fn paint_chip_state(p: &egui::Painter, rect: Rect, radius: u8, s: ChipState) {
    let base = if s.active { LINE_STRONG } else { LINE };
    let mut outline = base;
    if let Some(c) = s.countdown {
        let c = c.clamp(0.0, 1.0);
        let h = rect.height() * c;
        if h > 0.5 {
            let fill =
                Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - h), rect.max).shrink(1.0);
            let top = if h >= rect.height() - radius as f32 {
                radius.saturating_sub(1)
            } else {
                0
            };
            let cr = CornerRadius {
                nw: top,
                ne: top,
                sw: radius.saturating_sub(1),
                se: radius.saturating_sub(1),
            };
            p.rect_filled(fill, cr, theme::alpha(INK, 0.16));
        }
        outline = theme::mix(base, INK_3, c);
    }
    if s.pressed > 0.01 {
        p.rect_filled(
            rect.shrink(1.0),
            CornerRadius::same(radius.saturating_sub(1)),
            theme::alpha(ACCENT, 0.18 * s.pressed),
        );
        outline = theme::mix(outline, ACCENT, s.pressed);
    }
    p.rect_stroke(
        rect,
        CornerRadius::same(radius),
        Stroke::new(1.0, outline),
        StrokeKind::Inside,
    );
}

/// A laptop key cap: label top left, an optional caps sub-label bottom left.
pub fn keycap(
    ui: &Ui,
    rect: Rect,
    label: &str,
    sub: Option<&str>,
    sub_color: Option<Color32>,
    s: ChipState,
) {
    let p = ui.painter();
    paint_chip_state(p, rect, 7, s);
    let lit = s.pressed.max(s.countdown.unwrap_or(0.0) * 0.5);
    let c = if s.active {
        theme::mix(INK_2, INK, lit)
    } else {
        INK_4
    };
    // Sized for a 50 px key; smaller keys (the player's three-row keyboard
    // on a short window) scale padding and type so letter and degree never
    // touch.
    let k = (rect.height() / 50.0).clamp(0.6, 1.0);
    let pad = Vec2::new(9.0 * k, 7.0 * k);
    p.text(
        rect.left_top() + pad,
        Align2::LEFT_TOP,
        label,
        theme::font(theme::SANS_MEDIUM, 15.0 * k),
        c,
    );
    if let Some(sub) = sub {
        let sc = sub_color.unwrap_or(if s.active {
            theme::mix(INK_3, INK_2, lit)
        } else {
            INK_4
        });
        let mut font = ty::label();
        font.size *= k;
        p.text(
            rect.left_bottom() + Vec2::new(pad.x, -pad.y),
            Align2::LEFT_BOTTOM,
            sub.to_uppercase(),
            font,
            sc,
        );
    }
}

/// A small key chip with a centred label (row labels of the roll).
pub fn key_chip(ui: &Ui, rect: Rect, label: &str, s: ChipState) {
    let p = ui.painter();
    paint_chip_state(p, rect, 5, s);
    let c = if s.active {
        theme::mix(INK_2, INK, s.pressed.max(s.countdown.unwrap_or(0.0)))
    } else {
        INK_3
    };
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        theme::font(theme::SANS_MEDIUM, 13.0),
        c,
    );
}
