//! The data folder: `profile.json` (atomic writes) and `history.jsonl` (one
//! line per run). A profile that cannot be read is never overwritten: the
//! store goes read-only and the app shows a permanent note.

use crate::profile::{PROFILE_VERSION, Profile};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What [`Store::open`] found.
#[derive(Debug, PartialEq)]
pub enum Loaded {
    /// No profile yet: a fresh one (saved on the first change).
    Fresh,
    /// The saved profile.
    Existing,
    /// The file exists but cannot be used; saving is off. The text says why.
    Unreadable(String),
}

/// Reads and writes one learner's data folder.
pub struct Store {
    dir: PathBuf,
    read_only: Option<String>,
}

impl Store {
    /// Opens `dir` and loads the profile (fresh profiles start on `today`).
    pub fn open(dir: &Path, today: i64) -> (Store, Profile, Loaded) {
        let path = dir.join("profile.json");
        let (profile, loaded) = match fs::read(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                (Profile::new(today), Loaded::Fresh)
            }
            Err(e) => (
                Profile::new(today),
                Loaded::Unreadable(format!("cannot read {}: {e}", path.display())),
            ),
            Ok(bytes) => match serde_json::from_slice::<Profile>(&bytes) {
                Ok(p) if p.version > PROFILE_VERSION => (
                    Profile::new(today),
                    Loaded::Unreadable(format!(
                        "{} is from a newer version ({} > {PROFILE_VERSION})",
                        path.display(),
                        p.version
                    )),
                ),
                Ok(p) => (p, Loaded::Existing),
                Err(e) => (
                    Profile::new(today),
                    Loaded::Unreadable(format!("{} is not a valid profile: {e}", path.display())),
                ),
            },
        };
        let read_only = match &loaded {
            Loaded::Unreadable(why) => Some(why.clone()),
            _ => None,
        };
        (
            Store {
                dir: dir.to_path_buf(),
                read_only,
            },
            profile,
            loaded,
        )
    }

    /// The data folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Why saving is off, if it is.
    pub fn read_only(&self) -> Option<&str> {
        self.read_only.as_deref()
    }

    /// Saves the profile atomically. Refuses while read-only.
    pub fn save(&self, p: &Profile) -> Result<(), String> {
        if let Some(why) = &self.read_only {
            return Err(format!("not saved: {why}"));
        }
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(p).map_err(|e| e.to_string())?;
        write_atomic(&self.dir.join("profile.json"), &bytes).map_err(|e| e.to_string())
    }

    /// Appends one JSON line to `history.jsonl`. Refuses while read-only.
    pub fn append_history(&self, line: &serde_json::Value) -> Result<(), String> {
        if let Some(why) = &self.read_only {
            return Err(format!("not saved: {why}"));
        }
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let mut bytes = serde_json::to_vec(line).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("history.jsonl"))
            .and_then(|mut f| f.write_all(&bytes))
            .map_err(|e| e.to_string())
    }

    /// Every line of `history.jsonl` that parses, oldest first.
    pub fn history(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.dir.join("history.jsonl"))
            .map(|t| {
                t.lines()
                    .filter_map(|l| serde_json::from_str(l).ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Writes through a temp file in the same folder, syncs it, then renames.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("learnapp-store-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn fresh_then_saved_then_existing() {
        let dir = temp("roundtrip");
        let (store, mut p, loaded) = Store::open(&dir, 7);
        assert_eq!(loaded, Loaded::Fresh);
        p.runs = 3;
        store.save(&p).unwrap();
        let (_, again, loaded) = Store::open(&dir, 8);
        assert_eq!(loaded, Loaded::Existing);
        assert_eq!(again.runs, 3);
        assert_eq!(again.created_day, 7);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn broken_profile_is_never_overwritten() {
        let dir = temp("broken");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("profile.json"), b"{ not json").unwrap();
        let (store, p, loaded) = Store::open(&dir, 1);
        assert!(matches!(loaded, Loaded::Unreadable(_)));
        assert!(store.save(&p).is_err());
        assert!(store.append_history(&serde_json::json!({})).is_err());
        assert_eq!(fs::read(dir.join("profile.json")).unwrap(), b"{ not json");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn newer_profile_is_never_overwritten() {
        let dir = temp("newer");
        fs::create_dir_all(&dir).unwrap();
        let text = format!(r#"{{"version":{},"created_day":1}}"#, PROFILE_VERSION + 1);
        fs::write(dir.join("profile.json"), &text).unwrap();
        let (store, p, loaded) = Store::open(&dir, 1);
        assert!(matches!(loaded, Loaded::Unreadable(_)));
        assert!(store.save(&p).is_err());
        assert_eq!(fs::read_to_string(dir.join("profile.json")).unwrap(), text);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn history_lines() {
        let dir = temp("history");
        let (store, _, _) = Store::open(&dir, 1);
        store
            .append_history(&serde_json::json!({"run": 1}))
            .unwrap();
        store
            .append_history(&serde_json::json!({"run": 2}))
            .unwrap();
        let h = store.history();
        assert_eq!(h.len(), 2);
        assert_eq!(h[1]["run"], 2);
        fs::remove_dir_all(&dir).unwrap();
    }
}
