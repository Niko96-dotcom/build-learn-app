//! Command line: `learnapp [--data-dir PATH] [--trace] [--size WxH]
//! [--screen NAME] [--background] [--now YYYY-MM-DD] [--version] [--help]`.

use crate::screens::Screen;
use std::path::PathBuf;

/// Parsed command line.
#[derive(Clone, Debug, PartialEq)]
pub struct Cli {
    /// `--data-dir`: the learner data folder (verification uses an empty one).
    pub data_dir: Option<PathBuf>,
    /// `--trace` or `LEARNAPP_TRACE=1`: write `trace.jsonl` and `state.json`.
    pub trace: bool,
    /// `--size WxH` (default 1440×900).
    pub size: [f32; 2],
    /// `--screen NAME`: open this screen first. A test shortcut, not a user path.
    pub screen: Option<Screen>,
    /// `--background` or `LEARNAPP_NO_ACTIVATE=1`: verification mode. The app
    /// never activates itself, stays behind other windows and keeps painting.
    pub background: bool,
    /// `--now YYYY-MM-DD` or `LEARNAPP_NOW`: test clock for the learning
    /// calendar. Honoured only with `--trace` and `--data-dir`.
    pub now: Option<String>,
    /// `--version`.
    pub version: bool,
    /// `--help`.
    pub help: bool,
}

/// Usage line.
pub const USAGE: &str = "learnapp [--data-dir PATH] [--trace] [--size WxH] [--screen today|practice|progress|settings] [--background] [--now YYYY-MM-DD] [--version] [--help]";

impl Cli {
    /// Parses arguments (without the program name).
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
        let mut cli = Cli {
            data_dir: None,
            trace: false,
            size: [1440.0, 900.0],
            screen: None,
            background: false,
            now: None,
            version: false,
            help: false,
        };
        let mut args = args.into_iter();
        while let Some(a) = args.next() {
            let mut value = |flag: &str| args.next().ok_or_else(|| format!("{flag} needs a value"));
            match a.as_str() {
                "--data-dir" => cli.data_dir = Some(PathBuf::from(value("--data-dir")?)),
                "--trace" => cli.trace = true,
                "--background" => cli.background = true,
                "--size" => {
                    let v = value("--size")?;
                    let (w, h) = v
                        .split_once('x')
                        .ok_or_else(|| format!("--size {v}: expected WxH"))?;
                    let w: f32 = w.parse().map_err(|_| format!("--size {v}: bad width"))?;
                    let h: f32 = h.parse().map_err(|_| format!("--size {v}: bad height"))?;
                    cli.size = [w, h];
                }
                "--screen" => {
                    let v = value("--screen")?;
                    cli.screen =
                        Some(Screen::parse(&v).ok_or_else(|| format!("unknown screen {v}"))?);
                }
                "--now" => cli.now = Some(value("--now")?),
                "--version" => cli.version = true,
                "--help" | "-h" => cli.help = true,
                other => return Err(format!("unknown argument {other}\n{USAGE}")),
            }
        }
        Ok(cli)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, String> {
        Cli::parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn defaults_and_flags() {
        let c = parse(&[]).unwrap();
        assert_eq!(c.size, [1440.0, 900.0]);
        assert!(!c.trace && !c.background);
        let c = parse(&[
            "--data-dir",
            "/tmp/x",
            "--trace",
            "--size",
            "1080x720",
            "--screen",
            "practice",
        ])
        .unwrap();
        assert_eq!(c.data_dir, Some(PathBuf::from("/tmp/x")));
        assert!(c.trace);
        assert_eq!(c.size, [1080.0, 720.0]);
        assert_eq!(c.screen, Some(Screen::Practice));
    }

    #[test]
    fn errors_name_the_problem() {
        assert!(parse(&["--size", "big"]).unwrap_err().contains("WxH"));
        assert!(parse(&["--nope"]).unwrap_err().contains("--nope"));
        assert!(
            parse(&["--data-dir"])
                .unwrap_err()
                .contains("needs a value")
        );
    }
}
