//! Learnapp: the binary. Parses the command line, sets up background mode
//! for verification, and runs the app.

use learnapp_app::cli::{Cli, USAGE};
use learnapp_app::{app, dates, platform, trace};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() -> eframe::Result {
    let cli = match Cli::parse(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if cli.version {
        println!("learnapp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if cli.help {
        println!("{USAGE}");
        return Ok(());
    }
    let trace = cli.trace || std::env::var("LEARNAPP_TRACE").is_ok_and(|v| v == "1");
    let background =
        cli.background || std::env::var("LEARNAPP_NO_ACTIVATE").is_ok_and(|v| v == "1");
    if background {
        if !platform::begin_background() {
            eprintln!("background: could not keep the app from activating; not starting");
            std::process::exit(3);
        }
        eprintln!("background: on (no activation, window at the back, paints while covered)");
    }
    let data_dir = trace::resolve_data_dir(cli.data_dir.as_deref());
    // Test clock: only for traced runs on an explicit data dir, never on the real profile.
    let mut today = dates::today_local();
    if let Some(now) = cli
        .now
        .clone()
        .or_else(|| std::env::var("LEARNAPP_NOW").ok())
    {
        match dates::parse_day(&now) {
            Some(day) if trace && cli.data_dir.is_some() => {
                today = day;
                eprintln!("test clock: {now}");
            }
            Some(_) => eprintln!("test clock {now} ignored: needs --trace and --data-dir"),
            None => eprintln!("test clock {now} ignored: expected YYYY-MM-DD"),
        }
    }
    let _nap = platform::NoAppNap::begin();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Learnapp")
            .with_app_id("com.example.learnapp")
            .with_inner_size(cli.size)
            .with_min_inner_size([1080.0, 720.0])
            .with_fullsize_content_view(true)
            .with_title_shown(false)
            .with_titlebar_shown(false)
            .with_active(!background),
        ..Default::default()
    };
    let opts = app::Options {
        data_dir,
        trace,
        screen: cli.screen,
        today,
        background,
    };
    eframe::run_native(
        "Learnapp",
        options,
        Box::new(move |cc| {
            let app = app::LearnApp::new(cc, opts);
            Ok(if background {
                Box::new(Background::new(app))
            } else {
                Box::new(app)
            })
        }),
    )
}

/// Background mode around the app: asks eframe to run the ui and paint even
/// while the window is covered (eframe skips both for hidden windows, and
/// the app's runs, trace and accessibility tree live in the ui pass); tells
/// the app its window has focus (it never becomes key, so the only keys it
/// gets are the ones a driver posts to its pid, and play keys count only
/// with focus); reports frame rate and key events to stderr every 5 s.
///
/// While the window is fully covered nothing reaches the screen (wgpu skips
/// presenting), so frames are capped at 60/s instead of spinning, and
/// screenshots come from the app's own renderer: a driver drops
/// `<name>.request` into `LEARNAPP_SHOT_DIR`; the next frame is rendered
/// offscreen and written as `<name>.bmp`, and the request is renamed
/// `<name>.done`.
struct Background {
    app: app::LearnApp,
    launched: bool,
    frames: u32,
    since: Instant,
    last_frame: Instant,
    shot_dir: Option<PathBuf>,
    shot_poll: Instant,
}

/// Shortest frame while covered (60 frames/s).
const COVERED_FRAME: Duration = Duration::from_micros(16_667);

impl Background {
    fn new(app: app::LearnApp) -> Self {
        Background {
            app,
            launched: false,
            frames: 0,
            since: Instant::now(),
            last_frame: Instant::now(),
            shot_dir: std::env::var_os("LEARNAPP_SHOT_DIR").map(PathBuf::from),
            shot_poll: Instant::now(),
        }
    }

    /// Turns `<name>.request` files into screenshot commands (every 100 ms).
    fn poll_shots(&mut self, ctx: &egui::Context) {
        let Some(dir) = &self.shot_dir else { return };
        if self.shot_poll.elapsed() < Duration::from_millis(100) {
            return;
        }
        self.shot_poll = Instant::now();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "request") {
                continue;
            }
            let bmp = path.with_extension("bmp");
            if std::fs::rename(&path, path.with_extension("done")).is_err() {
                continue;
            }
            let ctx2 = ctx.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                egui::ScreenshotCallback::new(move |image| {
                    if let Err(e) = write_bmp(&bmp, &image) {
                        eprintln!("background: screenshot {}: {e}", bmp.display());
                    }
                    ctx2.request_repaint();
                }),
            ));
            ctx.request_repaint();
        }
    }
}

/// A 32-bit top-down BMP (sips and Preview read it).
fn write_bmp(path: &Path, image: &Arc<egui::ColorImage>) -> std::io::Result<()> {
    let [w, h] = image.size;
    let data_len = (w * h * 4) as u32;
    let mut out = Vec::with_capacity(54 + data_len as usize);
    out.extend(b"BM");
    out.extend((54 + data_len).to_le_bytes());
    out.extend(0u32.to_le_bytes());
    out.extend(54u32.to_le_bytes());
    out.extend(40u32.to_le_bytes());
    out.extend((w as i32).to_le_bytes());
    out.extend((-(h as i32)).to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(32u16.to_le_bytes());
    out.extend(0u32.to_le_bytes());
    out.extend(data_len.to_le_bytes());
    out.extend([0x13, 0x0B, 0, 0, 0x13, 0x0B, 0, 0]);
    out.extend([0u8; 8]);
    for c in &image.pixels {
        let [r, g, b, a] = c.to_srgba_unmultiplied();
        out.extend([b, g, r, a]);
    }
    let tmp = path.with_extension("bmp.tmp");
    std::fs::write(&tmp, out)?;
    std::fs::rename(&tmp, path)
}

impl eframe::App for Background {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.launched {
            self.launched = true;
            platform::background_after_launch();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
        if !ctx.input(|i| i.viewport().visible().unwrap_or(true)) {
            // This pass was logic only; the next one paints.
            ctx.request_repaint();
        }
        self.app.logic(ctx, frame);
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        raw_input.focused = true;
        self.app.raw_input_hook(ctx, raw_input);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        self.poll_shots(ui.ctx());
        if ui.input(|i| i.viewport().occluded != Some(false)) {
            // No vsync while covered (or not yet known): keep the frame rate
            // of a visible window instead of spinning.
            let spent = self.last_frame.elapsed();
            if spent < COVERED_FRAME {
                std::thread::sleep(COVERED_FRAME - spent);
            }
        }
        self.last_frame = Instant::now();
        self.frames += 1;
        let elapsed = self.since.elapsed();
        if elapsed >= Duration::from_secs(5) {
            let occluded = ui.input(|i| i.viewport().occluded);
            eprintln!(
                "background: {:.1} frames/s, occluded {:?}, other key events {} (play keys are counted in the trace)",
                self.frames as f64 / elapsed.as_secs_f64(),
                occluded,
                platform::key_events_seen()
            );
            self.frames = 0;
            self.since = Instant::now();
        }
    }

    fn on_exit(&mut self) {
        self.app.on_exit();
    }
}
