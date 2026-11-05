//! Visual language: colour, type, spacing, motion. Dark, calm, typographic.
//! No cards: structure comes from type, alignment, hairlines and space.
//!
//! This is the design system of the first app built with this skill (Pocket).
//! Keep it unchanged unless the learner asks for a new look: change usage, not
//! tokens. Rules: one accent, dark ground, no cards, no shadows, no gradients,
//! no glows, no cream.

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke};
use std::sync::Arc;

// Colour ---------------------------------------------------------------------

/// Window background.
pub const BG: Color32 = Color32::from_rgb(0x0A, 0x0A, 0x0B);
/// Hover wash for rows. Never a filled panel.
pub const WASH: Color32 = Color32::from_rgb(0x13, 0x13, 0x15);
/// Hairlines and beat lines.
pub const LINE: Color32 = Color32::from_rgb(0x1D, 0x1D, 0x21);
/// Bar lines and outlines.
pub const LINE_STRONG: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x31);
/// Primary text.
pub const INK: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF3);
/// Secondary text.
pub const INK_2: Color32 = Color32::from_rgb(0xA4, 0xA4, 0xAC);
/// Labels, targets.
pub const INK_3: Color32 = Color32::from_rgb(0x62, 0x62, 0x6A);
/// Disabled, faint marks.
pub const INK_4: Color32 = Color32::from_rgb(0x38, 0x38, 0x3E);
/// Signal colour: the playhead, the primary action, "live".
pub const ACCENT: Color32 = Color32::from_rgb(0xFF, 0x4D, 0x2E);
/// Hover state of the accent.
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xFF, 0x6E, 0x52);
/// Early hits.
pub const EARLY: Color32 = Color32::from_rgb(0x5C, 0xA4, 0xFF);
/// Late hits.
pub const LATE: Color32 = Color32::from_rgb(0xFF, 0xB5, 0x47);
/// Passed / tight.
pub const GOOD: Color32 = Color32::from_rgb(0x9E, 0xF0, 0x7A);

/// `c` with opacity `a` (0..1).
pub fn alpha(c: Color32, a: f32) -> Color32 {
    let a = a.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a * 255.0) as u8)
}

/// Linear mix of two colours, `t` = 0 → `a`, 1 → `b`.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// A 1 px hairline stroke.
pub fn hairline() -> Stroke {
    Stroke::new(1.0, LINE)
}

/// Colour for a signed timing offset (ms): blue early, amber late, ink on time.
pub fn offset_color(offset_ms: f64, good_ms: f64) -> Color32 {
    let t = (offset_ms.abs() / good_ms.max(1.0)).min(1.0) as f32;
    if offset_ms < 0.0 {
        mix(INK, EARLY, t)
    } else {
        mix(INK, LATE, t)
    }
}

// Type -----------------------------------------------------------------------

/// Geist Regular.
pub const SANS: &str = "sans";
/// Geist Light.
pub const SANS_LIGHT: &str = "sans-light";
/// Geist Medium.
pub const SANS_MEDIUM: &str = "sans-medium";
/// Geist SemiBold.
pub const SANS_SEMI: &str = "sans-semibold";
/// Geist Mono Regular.
pub const MONO: &str = "mono";
/// Geist Mono Light.
pub const MONO_LIGHT: &str = "mono-light";
/// Geist Mono Medium.
pub const MONO_MEDIUM: &str = "mono-medium";
/// Geist Pixel Square (big numerals).
pub const PIXEL: &str = "pixel";
/// Geist Pixel Circle.
pub const PIXEL_DOT: &str = "pixel-dot";

/// Registers the faces and their fallback chains.
pub fn install_fonts(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

/// The font set: each named family = its face, then egui's defaults (arrows,
/// symbols). Geist is first in Proportional, Geist Mono first in Monospace.
pub fn font_definitions() -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    let fonts: [(&str, &'static [u8]); 9] = [
        (
            SANS,
            include_bytes!("../../../assets/fonts/Geist-Regular.ttf"),
        ),
        (
            SANS_LIGHT,
            include_bytes!("../../../assets/fonts/Geist-Light.ttf"),
        ),
        (
            SANS_MEDIUM,
            include_bytes!("../../../assets/fonts/Geist-Medium.ttf"),
        ),
        (
            SANS_SEMI,
            include_bytes!("../../../assets/fonts/Geist-SemiBold.ttf"),
        ),
        (
            MONO,
            include_bytes!("../../../assets/fonts/GeistMono-Regular.ttf"),
        ),
        (
            MONO_LIGHT,
            include_bytes!("../../../assets/fonts/GeistMono-Light.ttf"),
        ),
        (
            MONO_MEDIUM,
            include_bytes!("../../../assets/fonts/GeistMono-Medium.ttf"),
        ),
        (
            PIXEL,
            include_bytes!("../../../assets/fonts/GeistPixel-Square.ttf"),
        ),
        (
            PIXEL_DOT,
            include_bytes!("../../../assets/fonts/GeistPixel-Circle.ttf"),
        ),
    ];
    let fallbacks = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (name, bytes) in fonts {
        defs.font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
        let mut chain = vec![name.to_owned()];
        chain.extend(fallbacks.iter().cloned());
        defs.families.insert(FontFamily::Name(name.into()), chain);
    }
    for (family, first) in [
        (FontFamily::Proportional, SANS),
        (FontFamily::Monospace, MONO),
    ] {
        defs.families
            .entry(family)
            .or_default()
            .insert(0, first.to_owned());
    }
    defs
}

/// A font of a named face.
pub fn font(family: &str, size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(family.into()))
}

/// Type scale.
pub mod ty {
    use super::*;
    /// Screen titles.
    pub fn display() -> FontId {
        font(SANS_SEMI, 44.0)
    }
    /// Big hero titles (Today, first start, summary).
    pub fn hero() -> FontId {
        font(SANS_SEMI, 52.0)
    }
    /// Section titles.
    pub fn title() -> FontId {
        font(SANS_SEMI, 26.0)
    }
    /// Row titles.
    pub fn heading() -> FontId {
        font(SANS_MEDIUM, 17.0)
    }
    /// Running text.
    pub fn body() -> FontId {
        font(SANS, 15.0)
    }
    /// Secondary text.
    pub fn small() -> FontId {
        font(SANS, 13.0)
    }
    /// Uppercase mono labels (draw with letter spacing).
    pub fn label() -> FontId {
        font(MONO, 11.0)
    }
    /// Mono numbers in rows.
    pub fn mono() -> FontId {
        font(MONO, 13.0)
    }
    /// Light mono numbers.
    pub fn number(size: f32) -> FontId {
        font(MONO_LIGHT, size)
    }
    /// Big readouts (tempo, score) in the dot-matrix face.
    pub fn readout(size: f32) -> FontId {
        font(PIXEL, size)
    }
}

// Space ----------------------------------------------------------------------

/// 4 px.
pub const S1: f32 = 4.0;
/// 8 px.
pub const S2: f32 = 8.0;
/// 12 px.
pub const S3: f32 = 12.0;
/// 16 px.
pub const S4: f32 = 16.0;
/// 24 px.
pub const S5: f32 = 24.0;
/// 32 px.
pub const S6: f32 = 32.0;
/// 48 px.
pub const S7: f32 = 48.0;
/// 64 px.
pub const S8: f32 = 64.0;
/// 96 px.
pub const S9: f32 = 96.0;

/// Content margin left/right.
pub const GUTTER: f32 = 72.0;
/// Height of the top bar (traffic lights + nav).
pub const TOPBAR: f32 = 64.0;

/// Applies the dark style to every theme of the context.
pub fn apply_style(ctx: &egui::Context) {
    ctx.all_styles_mut(|s| {
        s.visuals = egui::Visuals::dark();
        s.visuals.panel_fill = BG;
        s.visuals.window_fill = BG;
        s.visuals.extreme_bg_color = BG;
        s.visuals.faint_bg_color = WASH;
        s.visuals.override_text_color = Some(INK);
        s.visuals.selection.bg_fill = alpha(ACCENT, 0.35);
        s.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        s.visuals.widgets.noninteractive.bg_stroke = hairline();
        s.visuals.widgets.inactive.bg_fill = BG;
        s.visuals.widgets.inactive.weak_bg_fill = BG;
        s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, LINE_STRONG);
        s.visuals.widgets.hovered.bg_fill = WASH;
        s.visuals.widgets.hovered.weak_bg_fill = WASH;
        s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, INK_3);
        s.visuals.widgets.active.bg_fill = WASH;
        s.visuals.widgets.active.bg_stroke = Stroke::new(1.0, INK_2);
        s.visuals.text_cursor.stroke = Stroke::new(2.0, ACCENT);
        s.visuals.window_corner_radius = 0.into();
        s.visuals.menu_corner_radius = 4.into();
        s.visuals.popup_shadow = egui::epaint::Shadow::NONE;
        s.visuals.window_shadow = egui::epaint::Shadow::NONE;
        s.spacing.item_spacing = egui::vec2(S2, S2);
        s.spacing.button_padding = egui::vec2(S3, S2);
        s.spacing.scroll = egui::style::ScrollStyle::thin();
        s.text_styles.insert(egui::TextStyle::Body, ty::body());
        s.text_styles.insert(egui::TextStyle::Button, ty::body());
        s.text_styles.insert(egui::TextStyle::Small, ty::small());
        s.text_styles.insert(egui::TextStyle::Monospace, ty::mono());
        s.text_styles.insert(egui::TextStyle::Heading, ty::title());
        s.animation_time = 0.18;
    });
}

// Motion ---------------------------------------------------------------------

/// Ease-out cubic, 0..1 → 0..1.
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Ease-in-out cubic, 0..1 → 0..1.
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Spring-like overshoot for small reveals.
pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.40158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Exponential decay from 1 to 0: `half_life` seconds halve the value (key glow).
pub fn decay(age_s: f32, half_life: f32) -> f32 {
    0.5f32.powf(age_s.max(0.0) / half_life.max(1e-3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_and_alpha_clamp() {
        assert_eq!(mix(BG, INK, 0.0), BG);
        assert_eq!(mix(BG, INK, 2.0), INK);
        assert_eq!(alpha(INK, 1.5).a(), 255);
    }

    #[test]
    fn offset_colour_direction() {
        assert_eq!(offset_color(-60.0, 60.0), EARLY);
        assert_eq!(offset_color(60.0, 60.0), LATE);
        assert_eq!(offset_color(0.0, 60.0), INK);
    }

    #[test]
    fn geist_comes_first() {
        let defs = font_definitions();
        assert_eq!(defs.families[&FontFamily::Proportional][0], SANS);
        assert_eq!(defs.families[&FontFamily::Monospace][0], MONO);
        assert_eq!(defs.families[&FontFamily::Name(PIXEL.into())][0], PIXEL);
    }

    #[test]
    fn decay_halves() {
        assert!((decay(0.09, 0.09) - 0.5).abs() < 1e-4);
        assert_eq!(decay(-1.0, 0.09), 1.0);
    }

    #[test]
    fn easing_ends() {
        for f in [ease_out_cubic, ease_in_out, ease_out_back] {
            assert!(f(0.0).abs() < 1e-5 && (f(1.0) - 1.0).abs() < 1e-5);
        }
    }
}
