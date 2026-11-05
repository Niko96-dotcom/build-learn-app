//! The data folder location and trace mode: `trace.jsonl` gets one JSON line
//! per event, `state.json` is rewritten on every change. Both exist only with
//! `--trace` / `LEARNAPP_TRACE=1`. They are the verification interface: the
//! driver reads them, and a screenshot is never proof without them.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

/// The data folder: `--data-dir`, else `LEARNAPP_DATA_DIR`, else
/// `~/Library/Application Support/Learnapp`.
pub fn resolve_data_dir(arg: Option<&Path>) -> PathBuf {
    if let Some(p) = arg {
        return p.to_path_buf();
    }
    if let Some(p) = std::env::var_os("LEARNAPP_DATA_DIR").filter(|p| !p.is_empty()) {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join("Library")
        .join("Application Support")
        .join("Learnapp")
}

/// Trace writer. A disabled tracer touches no files.
pub struct Tracer {
    root: PathBuf,
    start: Instant,
    file: Option<Mutex<File>>,
    last_state: Mutex<Option<serde_json::Value>>,
}

impl Tracer {
    /// Opens `trace.jsonl` in `root` when `enabled`; behaves as disabled if it cannot.
    pub fn new(root: &Path, enabled: bool) -> Tracer {
        let file = enabled
            .then(|| {
                fs::create_dir_all(root).ok()?;
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(root.join("trace.jsonl"))
                    .ok()
            })
            .flatten()
            .map(Mutex::new);
        Tracer {
            root: root.to_path_buf(),
            start: Instant::now(),
            file,
            last_state: Mutex::new(None),
        }
    }

    /// True when trace mode is on.
    pub fn enabled(&self) -> bool {
        self.file.is_some()
    }

    /// Appends `{"kind", "t_ms", "data"}` (`t_ms` = ms since the tracer started).
    pub fn event(&self, kind: &str, data: serde_json::Value) {
        let Some(file) = &self.file else { return };
        let t_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        let line = serde_json::json!({ "kind": kind, "t_ms": t_ms, "data": data });
        let Ok(mut bytes) = serde_json::to_vec(&line) else {
            return;
        };
        bytes.push(b'\n');
        if let Ok(mut f) = file.lock() {
            let _ = f.write_all(&bytes);
        }
    }

    /// Rewrites `state.json` atomically when it changed.
    pub fn state(&self, data: serde_json::Value) {
        if self.file.is_none() {
            return;
        }
        let Ok(mut last) = self.last_state.lock() else {
            return;
        };
        if last.as_ref() == Some(&data) {
            return;
        }
        if let Ok(bytes) = serde_json::to_vec_pretty(&data) {
            let path = self.root.join("state.json");
            let tmp = path.with_extension("json.tmp");
            if fs::write(&tmp, bytes).is_ok() && fs::rename(&tmp, &path).is_ok() {
                *last = Some(data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("learnapp-trace-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn data_dir_argument_wins() {
        assert_eq!(resolve_data_dir(Some(Path::new("/x"))), PathBuf::from("/x"));
    }

    #[test]
    fn disabled_tracer_writes_nothing() {
        let root = temp("off");
        let t = Tracer::new(&root, false);
        t.event("key", serde_json::json!({}));
        t.state(serde_json::json!({}));
        assert!(!root.exists());
    }

    #[test]
    fn events_and_state() {
        let root = temp("on");
        let t = Tracer::new(&root, true);
        t.event("screen", serde_json::json!({ "screen": "today" }));
        t.event("hit", serde_json::json!({ "code": 3, "host_ns": 5 }));
        t.state(serde_json::json!({ "screen": "practice" }));
        let lines: Vec<serde_json::Value> = fs::read_to_string(root.join("trace.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1]["kind"], "hit");
        assert_eq!(lines[1]["data"]["host_ns"], 5);
        let state: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("state.json")).unwrap()).unwrap();
        assert_eq!(state["screen"], "practice");
        fs::remove_dir_all(&root).unwrap();
    }
}
