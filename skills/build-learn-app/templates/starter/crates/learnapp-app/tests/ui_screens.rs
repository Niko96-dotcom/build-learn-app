//! UI tests: the real app rendered offscreen with wgpu at 1440 × 900, driven
//! through its accessibility labels. No native key monitor
//! (`LEARNAPP_NO_NATIVE_INPUT`). With `LEARNAPP_RENDER_DIR=<dir>` every
//! screen is also saved there as a PNG for a design review by eye.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use learnapp_app::app::{LearnApp, Options};
use learnapp_app::screens::Screen;
use learnapp_app::screens::practice::Phase;
use std::path::PathBuf;

/// The harnesses share wgpu state: run them one at a time.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn harness(name: &str, screen: Screen) -> Harness<'static, LearnApp> {
    // SAFETY: every test sets the same value before the app reads it.
    unsafe { std::env::set_var("LEARNAPP_NO_NATIVE_INPUT", "1") };
    let data_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("ui")
        .join(name);
    let _ = std::fs::remove_dir_all(&data_dir);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .wgpu()
        .build_eframe(move |cc| {
            LearnApp::new(
                cc,
                Options {
                    data_dir,
                    trace: true,
                    screen: Some(screen),
                    // 2026-11-05: fixed content, no machine date in the pictures.
                    today: 20_762,
                    background: false,
                },
            )
        });
    h.run_steps(40);
    h
}

fn render(h: &mut Harness<'static, LearnApp>, name: &str) {
    if let Some(dir) = std::env::var_os("LEARNAPP_RENDER_DIR") {
        let img = h.render().expect("render");
        std::fs::create_dir_all(&dir).expect("render dir");
        img.save(PathBuf::from(dir).join(format!("{name}.png")))
            .expect("save png");
    }
}

#[test]
fn every_screen_renders_with_its_labels() {
    let _g = serial();
    for (screen, labels) in [
        (
            Screen::Today,
            vec!["Start session", "Today", "Practice", "Progress", "Settings"],
        ),
        (Screen::Practice, vec!["Start"]),
        (Screen::Progress, vec![]),
        (
            Screen::Settings,
            vec!["Reduce motion", "Level", "Reset progress"],
        ),
    ] {
        let mut h = harness(screen.id(), screen);
        for l in labels {
            let _ = h.get_by_label(l);
        }
        render(&mut h, screen.id());
    }
}

#[test]
fn nav_and_start_move_the_app() {
    let _g = serial();
    let mut h = harness("nav", Screen::Today);
    h.get_by_label("Progress").click();
    h.run_steps(5);
    assert_eq!(h.state().screen(), Screen::Progress);
    h.get_by_label("Practice").click();
    h.run_steps(5);
    h.get_by_label("Start").click();
    h.run_steps(5);
    assert_eq!(h.state().phase(), Phase::CountIn);
    render(&mut h, "practice-count-in");
    h.get_by_label("Leave session").click();
    h.run_steps(5);
    assert_eq!(h.state().screen(), Screen::Today);
    assert_eq!(h.state().phase(), Phase::Ready);
}

#[test]
fn reset_needs_two_clicks() {
    let _g = serial();
    let mut h = harness("reset", Screen::Settings);
    h.get_by_label("Reset progress").click();
    h.run_steps(5);
    let _ = h.get_by_label("Move to Trash");
    h.get_by_label("Keep it").click();
    h.run_steps(5);
    let _ = h.get_by_label("Reset progress");
}
