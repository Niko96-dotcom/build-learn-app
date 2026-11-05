//! The app: state, the frame loop, actions, persistence and the trace.
//!
//! The demo exercise ("Steady pulse") is a placeholder for the real content:
//! eight targets on two keys after a count-in, scored by host-time key
//! stamps. It shows the contracts every learning app here keeps:
//! - every key press carries its hardware timestamp (`input.rs`);
//! - a run publishes a `transport` trace event with its zero time and the
//!   correct answers, so the driver can answer like a learner;
//! - results are measured facts; "done for today" only after a finished run;
//! - nothing is saved over a profile that cannot be read.

use crate::dates;
use crate::input::{self, KeyInput, KeySet};
use crate::screens::practice::{LaneKey, Phase, PracticeView};
use crate::screens::progress::{ProgressView, RunRow};
use crate::screens::settings::SettingsView;
use crate::screens::today::{PlanRow, TodayView};
use crate::screens::{self, Action, Frame, Screen};
use crate::theme::{self, ACCENT, BG, INK, INK_2, INK_3};
use crate::trace::Tracer;
use crate::widgets;
use egui::{Align2, Pos2, Rect, Ui, Vec2};
use learnapp_core::schedule::Stage;
use learnapp_core::score::{self, Gate, Hit, Summary, Target, Windows};
use learnapp_core::{Loaded, Outcome, Profile, Store};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

/// Start-up options.
#[derive(Clone, Debug)]
pub struct Options {
    /// The data folder.
    pub data_dir: PathBuf,
    /// Trace mode (`trace.jsonl`, `state.json`).
    pub trace: bool,
    /// Open this screen first (test shortcut).
    pub screen: Option<Screen>,
    /// Day number of "today" (local time or the test clock).
    pub today: i64,
    /// Background mode (verification).
    pub background: bool,
}

/// The demo item.
const ITEM_ID: &str = "steady-pulse";
const ITEM_TITLE: &str = "Steady pulse";
const BPM: f64 = 100.0;
const COUNT_IN: usize = 4;
/// F and J, the keys with the bumps.
const KEYS: [(u16, &str); 2] = [(0x03, "F"), (0x26, "J")];
const TARGETS: usize = 8;

struct Run {
    zero_ns: u64,
    beat_ms: f64,
    targets: Vec<Target>,
    hits: Vec<Hit>,
    phase: Phase,
    result: Option<(Summary, String, bool)>,
}

impl Run {
    fn now_ms(&self, now_ns: u64) -> f64 {
        (now_ns as f64 - self.zero_ns as f64) / 1e6
    }

    fn end_ms(&self) -> f64 {
        self.targets.last().map(|t| t.ms).unwrap_or(0.0)
            + Windows::BEGINNER.counted
            + self.beat_ms * 0.5
    }
}

/// The app.
pub struct LearnApp {
    store: Store,
    profile: Profile,
    tracer: Tracer,
    input: KeyInput,
    screen: Screen,
    since: f64,
    t: f64,
    today: i64,
    run: Option<Run>,
    glow: HashMap<u16, f64>,
    reset_asked: bool,
    note: Option<String>,
    background: bool,
}

impl LearnApp {
    /// Builds the app on the eframe context.
    pub fn new(cc: &eframe::CreationContext<'_>, o: Options) -> LearnApp {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_style(&cc.egui_ctx);
        let (store, profile, loaded) = Store::open(&o.data_dir, o.today);
        let note = match &loaded {
            Loaded::Unreadable(why) => Some(format!("Your progress is not saved: {why}")),
            _ => None,
        };
        let input = KeyInput::new(KeySet::from_codes(&KEYS.map(|k| k.0)));
        let tracer = Tracer::new(&o.data_dir, o.trace);
        tracer.event(
            "start",
            json!({
                "version": env!("CARGO_PKG_VERSION"),
                "data_dir": o.data_dir.display().to_string(),
                "input": if input.is_native() { "native" } else { "egui" },
                // "none": this build has no audio engine. An engine reports "off" when
                // LEARNAPP_NO_AUDIO=1 (verification) and "on" otherwise.
                "audio": "none",
                "background": o.background,
                "today": o.today,
                "profile": match loaded { Loaded::Fresh => "fresh", Loaded::Existing => "existing", Loaded::Unreadable(_) => "unreadable" },
            }),
        );
        let mut app = LearnApp {
            store,
            profile,
            tracer,
            input,
            screen: Screen::Today,
            since: 0.0,
            t: 0.0,
            today: o.today,
            run: None,
            glow: HashMap::new(),
            reset_asked: false,
            note,
            background: o.background,
        };
        app.go(o.screen.unwrap_or(Screen::Today));
        app
    }

    /// The current screen.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// The current run phase (Ready without a run).
    pub fn phase(&self) -> Phase {
        self.run.as_ref().map(|r| r.phase).unwrap_or_default()
    }

    fn go(&mut self, s: Screen) {
        if s != self.screen || self.since == 0.0 {
            if self.screen == Screen::Practice && s != Screen::Practice {
                self.leave_run();
            }
            self.screen = s;
            self.since = self.t;
            self.reset_asked = false;
            self.tracer.event("screen", json!({ "screen": s.id() }));
        }
    }

    fn start_run(&mut self) {
        self.go(Screen::Practice);
        let beat_ms = 60_000.0 / BPM;
        let zero_ns = input::host_now_ns() + (COUNT_IN as f64 * beat_ms * 1e6) as u64;
        let targets: Vec<Target> = (0..TARGETS)
            .map(|i| Target {
                ms: i as f64 * beat_ms,
                key: KEYS[i % 2].0,
            })
            .collect();
        self.tracer.event(
            "transport",
            json!({
                "scored": true,
                "item": ITEM_ID,
                "zero_host_ns": zero_ns,
                "beat_ms": beat_ms,
                "count_in": COUNT_IN,
                "targets": targets.iter().map(|t| json!({
                    "ms": t.ms,
                    "code": t.key,
                    "key": KEYS.iter().find(|k| k.0 == t.key).map(|k| k.1.to_lowercase()),
                    "hold_ms": 80,
                })).collect::<Vec<_>>(),
            }),
        );
        self.run = Some(Run {
            zero_ns,
            beat_ms,
            targets,
            hits: Vec::new(),
            phase: Phase::CountIn,
            result: None,
        });
    }

    fn leave_run(&mut self) {
        if let Some(r) = &self.run
            && matches!(r.phase, Phase::CountIn | Phase::Playing)
        {
            self.tracer
                .event("run_left", json!({ "item": ITEM_ID, "hits": r.hits.len() }));
        }
        self.run = None;
        self.input.release_all();
    }

    /// Key events of this frame: glow, and hits while a run is open.
    fn take_keys(&mut self, ctx: &egui::Context) {
        self.input.feed_egui(ctx);
        let active = self.screen == Screen::Practice
            && self
                .run
                .as_ref()
                .is_some_and(|r| matches!(r.phase, Phase::CountIn | Phase::Playing));
        self.input.set_active(active);
        for ev in self.input.drain() {
            if !ev.down {
                continue;
            }
            self.glow.insert(ev.code, self.t);
            if let Some(run) = &mut self.run
                && matches!(run.phase, Phase::CountIn | Phase::Playing)
            {
                let ms = run.now_ms(ev.host_ns);
                run.hits.push(Hit { ms, key: ev.code });
                self.tracer.event(
                    "hit",
                    json!({ "code": ev.code, "host_ns": ev.host_ns, "ms": ms }),
                );
            }
        }
    }

    /// Moves the run on by the clock and scores it at the end.
    fn update_run(&mut self) {
        let now = input::host_now_ns();
        let Some(run) = &mut self.run else { return };
        let ms = run.now_ms(now);
        run.phase = match run.phase {
            Phase::CountIn if ms >= -run.beat_ms * 0.5 => Phase::Playing,
            Phase::Playing if ms >= run.end_ms() => Phase::Summary,
            p => p,
        };
        if run.phase == Phase::Summary && run.result.is_none() {
            let s = score::score(&run.targets, &run.hits, Windows::BEGINNER);
            let passed = Gate::BEGINNER.passed(&s);
            let verdict = score::verdict(&s, Gate::BEGINNER);
            run.result = Some((s.clone(), verdict.clone(), passed));
            self.finish(s, verdict, passed);
        }
    }

    /// A finished run: profile, history, trace. The only place that marks a day.
    fn finish(&mut self, s: Summary, verdict: String, passed: bool) {
        let outcome = if passed { Outcome::Pass } else { Outcome::Fail };
        let today = self.today;
        let mem = self.profile.items.entry(ITEM_ID.to_string()).or_default();
        let kind =
            if matches!(mem.stage, Stage::ColdPending | Stage::Known) && mem.cold(today, outcome) {
                "cold"
            } else {
                mem.lesson(today, outcome);
                "lesson"
            };
        let stage = format!("{:?}", mem.stage);
        self.profile.runs += 1;
        self.profile.practice_days.insert(today);
        let line = json!({
            "day": today, "item": ITEM_ID, "kind": kind, "passed": passed,
            "accuracy": s.accuracy, "bias_ms": s.bias_ms, "spread_ms": s.spread_ms,
            "matched": s.matched, "targets": s.targets, "missed": s.missed, "extras": s.extras,
        });
        let saved = self
            .store
            .save(&self.profile)
            .and_then(|_| self.store.append_history(&line));
        match saved {
            Ok(()) => self
                .tracer
                .event("saved", json!({ "runs": self.profile.runs })),
            Err(why) => {
                self.tracer.event("save_failed", json!({ "why": why }));
                self.note = Some(format!("Your progress is not saved: {why}"));
            }
        }
        self.tracer.event(
            "run_end",
            json!({ "item": ITEM_ID, "kind": kind, "stage": stage, "passed": passed, "verdict": verdict, "summary": s }),
        );
    }

    fn apply(&mut self, a: Action) {
        match a {
            Action::Go(s) => self.go(s),
            Action::Start => self.start_run(),
            Action::Leave => {
                self.leave_run();
                self.go(Screen::Today);
            }
            Action::ReduceMotion(on) => {
                self.profile.settings.reduce_motion = on;
                self.save_settings();
            }
            Action::Level(l) => {
                self.profile.settings.level = l;
                self.save_settings();
            }
            Action::ResetAsk => self.reset_asked = true,
            Action::ResetCancel => self.reset_asked = false,
            Action::ResetConfirm => self.reset(),
        }
    }

    fn save_settings(&mut self) {
        let s = &self.profile.settings;
        self.tracer.event(
            "settings",
            json!({ "reduce_motion": s.reduce_motion, "level": s.level }),
        );
        if let Err(why) = self.store.save(&self.profile) {
            self.note = Some(format!("Your progress is not saved: {why}"));
        }
    }

    /// Moves profile and history to the Trash (never deletes) and starts fresh.
    fn reset(&mut self) {
        self.reset_asked = false;
        let mut moved = Vec::new();
        for name in ["profile.json", "history.jsonl"] {
            let p = self.store.dir().join(name);
            if p.exists() {
                match crate::platform::trash(&p) {
                    Ok(()) => moved.push(name),
                    Err(why) => {
                        self.note = Some(format!("Reset stopped: {why}"));
                        self.tracer.event("reset_failed", json!({ "why": why }));
                        return;
                    }
                }
            }
        }
        let dir = self.store.dir().to_path_buf();
        let (store, profile, _) = Store::open(&dir, self.today);
        self.store = store;
        self.profile = profile;
        self.note = None;
        self.tracer
            .event("reset", json!({ "moved_to_trash": moved }));
    }

    fn frame(&self) -> Frame {
        Frame {
            t: self.t,
            since: self.since,
            reduce_motion: self.profile.settings.reduce_motion,
        }
    }

    fn last_run(&self) -> Option<serde_json::Value> {
        self.store.history().into_iter().last()
    }

    fn today_view(&self) -> TodayView {
        let due = self.profile.due(self.today).len();
        TodayView {
            date: dates::long_name(self.today),
            done_today: self.profile.practice_days.contains(&self.today),
            plan: vec![
                PlanRow {
                    kind: "Warm-up",
                    title: ITEM_TITLE.into(),
                    minutes: 2,
                },
                PlanRow {
                    kind: "Review",
                    title: if due == 0 {
                        "Nothing due".into()
                    } else {
                        format!("{due} due")
                    },
                    minutes: 5,
                },
                PlanRow {
                    kind: "New",
                    title: "Next item".into(),
                    minutes: 8,
                },
            ],
            spread_ms: self.last_run().and_then(|l| l["spread_ms"].as_f64()),
            ring: self.profile.week_ring(self.today),
            week: (
                self.profile.days_this_week(self.today),
                learnapp_core::profile::WEEKLY_TARGET,
            ),
            note: self.note.clone(),
        }
    }

    fn practice_view(&self) -> PracticeView {
        let now = input::host_now_ns();
        let row_of = |code: u16| KEYS.iter().position(|k| k.0 == code).unwrap_or(0);
        let (phase, now_ms, targets, hits, summary, verdict, beat_ms) = match &self.run {
            None => (
                Phase::Ready,
                -60_000.0 / BPM * COUNT_IN as f64,
                Vec::new(),
                Vec::new(),
                None,
                None,
                60_000.0 / BPM,
            ),
            Some(r) => {
                let ms = match r.phase {
                    Phase::Summary => r.end_ms(),
                    _ => r.now_ms(now),
                };
                let scored = r.result.as_ref().map(|x| &x.0);
                let hits = r
                    .hits
                    .iter()
                    .map(|h| {
                        let off = scored.and_then(|s| {
                            r.targets
                                .iter()
                                .zip(&s.offsets)
                                .filter(|(t, o)| {
                                    t.key == h.key
                                        && o.is_some_and(|o| (t.ms + o - h.ms).abs() < 1e-6)
                                })
                                .map(|(_, o)| *o)
                                .next()
                                .flatten()
                        });
                        let live = if scored.is_none() {
                            r.targets
                                .iter()
                                .filter(|t| t.key == h.key)
                                .map(|t| h.ms - t.ms)
                                .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                                .filter(|o| o.abs() <= Windows::BEGINNER.counted)
                        } else {
                            off
                        };
                        (h.ms, row_of(h.key), live)
                    })
                    .collect();
                (
                    r.phase,
                    ms,
                    r.targets.iter().map(|t| (t.ms, row_of(t.key))).collect(),
                    hits,
                    r.result.as_ref().map(|x| x.0.clone()),
                    r.result.as_ref().map(|x| x.1.clone()),
                    r.beat_ms,
                )
            }
        };
        PracticeView {
            meta: format!("Warm-up · {} · {} targets", ITEM_TITLE, TARGETS),
            title: ITEM_TITLE.into(),
            cue: "Land each press on the beat so the line and the mark meet.".into(),
            phase,
            bpm: BPM,
            beat_ms,
            now_ms,
            targets,
            hits,
            good_ms: Windows::BEGINNER.good,
            keys: KEYS
                .iter()
                .map(|(code, label)| LaneKey {
                    code: *code,
                    label: label.to_string(),
                    glow: self
                        .glow
                        .get(code)
                        .map(|t0| theme::decay((self.t - t0) as f32, 0.09))
                        .unwrap_or(0.0),
                })
                .collect(),
            summary,
            verdict,
        }
    }

    fn progress_view(&self) -> ProgressView {
        ProgressView {
            runs: self
                .store
                .history()
                .iter()
                .map(|l| RunRow {
                    day: dates::short_name(l["day"].as_i64().unwrap_or(self.today)),
                    accuracy: l["accuracy"].as_f64().unwrap_or(0.0),
                    spread_ms: l["spread_ms"].as_f64().unwrap_or(0.0),
                    passed: l["passed"].as_bool().unwrap_or(false),
                })
                .collect(),
            days: self.profile.practice_days.len(),
        }
    }

    fn settings_view(&self) -> SettingsView {
        let home = std::env::var("HOME").unwrap_or_default();
        let dir = self.store.dir().display().to_string();
        SettingsView {
            reduce_motion: self.profile.settings.reduce_motion,
            level: self.profile.settings.level,
            data_dir: if !home.is_empty() && dir.starts_with(&home) {
                dir.replacen(&home, "~", 1)
            } else {
                dir
            },
            reset_asked: self.reset_asked,
        }
    }

    fn write_state(&self) {
        if !self.tracer.enabled() {
            return;
        }
        let last = self
            .run
            .as_ref()
            .and_then(|r| r.result.as_ref())
            .map(|(s, v, p)| {
                json!({ "accuracy": s.accuracy, "bias_ms": s.bias_ms, "spread_ms": s.spread_ms,
                    "matched": s.matched, "targets": s.targets, "missed": s.missed,
                    "extras": s.extras, "verdict": v, "passed": p })
            });
        self.tracer.state(json!({
            "screen": self.screen.id(),
            "phase": self.phase().id(),
            "item": ITEM_ID,
            "today": self.today,
            "runs": self.profile.runs,
            "done_today": self.profile.practice_days.contains(&self.today),
            "days_this_week": self.profile.days_this_week(self.today),
            "item_stage": self.profile.items.get(ITEM_ID).map(|m| format!("{:?}", m.stage)),
            "reduce_motion": self.profile.settings.reduce_motion,
            "reset_asked": self.reset_asked,
            "saving": self.store.read_only().is_none(),
            "note": self.note,
            "background": self.background,
            "last": last,
        }));
    }

    fn shortcuts(&self, ui: &Ui) -> Option<Action> {
        let (space, esc) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Space),
                i.key_pressed(egui::Key::Escape),
            )
        });
        match (self.screen, self.phase()) {
            (Screen::Today, _) if space => Some(Action::Start),
            (Screen::Practice, Phase::Ready | Phase::Summary) if space => Some(Action::Start),
            (Screen::Practice, Phase::CountIn | Phase::Playing) if esc => Some(Action::Leave),
            _ => None,
        }
    }

    fn top_bar(&self, ui: &mut Ui, rect: Rect) -> Option<Action> {
        let drag = ui.interact(
            rect,
            ui.scope_id().with("drag"),
            egui::Sense::click_and_drag(),
        );
        if drag.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        let y = rect.top() + 34.0;
        let wr = widgets::text_at(
            ui,
            Pos2::new(rect.left() + 92.0, y),
            Align2::LEFT_CENTER,
            "learnapp",
            theme::font(theme::SANS_SEMI, 17.0),
            INK,
        );
        ui.painter()
            .circle_filled(Pos2::new(wr.right() + 3.6, y + 4.0), 2.6, ACCENT);
        let font = theme::font(theme::SANS_MEDIUM, 14.0);
        let mut x = rect.right() - theme::GUTTER;
        let mut action = None;
        for (s, label) in Screen::NAV.iter().rev() {
            let g = widgets::galley(ui, label, font.clone(), INK, 0.0);
            let r = Rect::from_min_size(Pos2::new(x - g.size().x, y - g.size().y / 2.0), g.size());
            x -= g.size().x + 28.0;
            let resp = ui.interact(
                r.expand(6.0),
                ui.scope_id().with(("nav", *label)),
                egui::Sense::click(),
            );
            resp.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::Tab, true, *s == self.screen, *label)
            });
            let hover = ui
                .ctx()
                .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.12);
            let active = *s == self.screen;
            let c = if active {
                INK
            } else {
                theme::mix(INK_3, INK_2, hover)
            };
            ui.painter()
                .galley(r.min, widgets::galley(ui, label, font.clone(), c, 0.0), c);
            let under = ui
                .ctx()
                .animate_bool_with_time(resp.id.with("u"), active, 0.22);
            if under > 0.01 {
                let w = r.width() * theme::ease_out_cubic(under);
                ui.painter().line_segment(
                    [
                        Pos2::new(r.left(), r.bottom() + 6.0),
                        Pos2::new(r.left() + w, r.bottom() + 6.0),
                    ],
                    egui::Stroke::new(1.5, ACCENT),
                );
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                action = Some(Action::Go(*s));
            }
        }
        action
    }

    fn animating(&self) -> bool {
        self.frame().entering()
            || self.run.as_ref().is_some_and(|r| r.phase != Phase::Summary)
            || self.glow.values().any(|t0| self.t - t0 < 0.6)
    }
}

impl eframe::App for LearnApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.t = ui.input(|i| i.time);
        if self.since == 0.0 {
            self.since = self.t;
        }
        let ctx = ui.ctx().clone();
        self.take_keys(&ctx);
        self.update_run();
        let full = ui.max_rect();
        ui.painter().rect_filled(full, 0.0, BG);
        let content = Rect::from_min_max(
            Pos2::new(
                full.left() + theme::GUTTER,
                full.top() + theme::TOPBAR + theme::S6,
            ),
            Pos2::new(full.right() - theme::GUTTER, full.bottom() - theme::S6),
        );
        let mut actions: Vec<Action> = self.shortcuts(ui).into_iter().collect();
        let f = self.frame();
        let screen_action = ui
            .scope_builder(egui::UiBuilder::new().max_rect(full), |ui| {
                match self.screen {
                    Screen::Today => screens::today::show(ui, content, &f, &self.today_view()),
                    Screen::Practice => {
                        screens::practice::show(ui, content, &f, &self.practice_view())
                    }
                    Screen::Progress => {
                        screens::progress::show(ui, content, &f, &self.progress_view())
                    }
                    Screen::Settings => {
                        screens::settings::show(ui, content, &f, &self.settings_view())
                    }
                }
            })
            .inner;
        actions.extend(screen_action);
        actions.extend(self.top_bar(
            ui,
            Rect::from_min_size(full.min, Vec2::new(full.width(), theme::TOPBAR)),
        ));
        for a in actions {
            self.apply(a);
        }
        self.write_state();
        if self.animating() {
            ui.ctx().request_repaint();
        } else {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    fn on_exit(&mut self) {
        self.leave_run();
        self.tracer.event("exit", json!({}));
    }
}
