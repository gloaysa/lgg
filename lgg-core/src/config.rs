use anyhow::{Context, Result};
use chrono::NaiveTime;
use directories::BaseDirs;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::keywords::Keywords;

#[derive(Debug, Clone)]
pub struct Config {
    /// Absolute directory where daily Markdown files live.
    pub journal_dir: PathBuf,
    /// Preferred editor name/binary (e.g. hx for Helix). Optional; the CLI will fall back to $VISUAL/$EDITOR.
    pub editor: Option<String>,
    /// Entries will be created at this time if you supply a date but not specific time (e.g. `yesterday:`).
    /// Valid format is "%H:%M" (e.g. 08:40 or 16:33). Default is 21:00.
    pub default_time: NaiveTime,
    pub date_format: String,
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    journal_dir: Option<PathBuf>,
    editor: Option<String>,
    default_time: Option<String>,
    pub date_format: Option<String>,
    /// Optional table:
    /// [synonyms]
    /// ytd = "yesterday"
    /// ayer = "yesterday"
    synonyms: Option<HashMap<String, String>>,
}

impl Config {
    /// Public entrypoint: load config from disk (first XDG path, then native), apply defaults,
    /// and extend the global Keywords registry with user-defined synonyms if present.
    pub fn load() -> Result<Self> {
        let file_config = Self::read_file_config().unwrap_or_else(|_| FileConfig {
            journal_dir: None,
            editor: None,
            default_time: None,
            synonyms: None,
            date_format: None,
        });

        let default_time = file_config
            .default_time
            .as_deref()
            .and_then(|time| Self::parse_default_time(&time))
            .unwrap_or_else(Self::default_fallback_time);

        let date_format = file_config
            .date_format
            .unwrap_or_else(|| "%A, %d %b %Y".to_string());

        let journal_dir = file_config
            .journal_dir
            .unwrap_or_else(Self::default_journal_dir);

        // Extend global keyword registry once at startup.
        Self::load_synonyms(&file_config.synonyms);

        Ok(Self {
            journal_dir,
            editor: file_config.editor,
            default_time,
            date_format,
        })
    }

    /// Default fallback time when user didn’t set `default_time` in config.
    fn default_fallback_time() -> NaiveTime {
        NaiveTime::from_hms_opt(21, 0, 0).expect("valid time")
    }

    /// Parse a "%H:%M" string into NaiveTime.
    fn parse_default_time(time: &str) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(time, "%H:%M").ok()
    }

    /// Default journal root: `{data_dir}/lgg`
    /// - macOS:   `~/Library/Application Support/lgg`
    /// - Linux:   `$XDG_DATA_HOME/lgg` or `~/.local/share/lgg`
    /// - Windows: `%APPDATA%\lgg`
    fn default_journal_dir() -> PathBuf {
        if let Some(base) = BaseDirs::new() {
            let mut p = base.data_dir().to_path_buf();
            p.push("lgg");
            p
        } else {
            PathBuf::from("./lgg")
        }
    }

    fn config_file_paths() -> Vec<PathBuf> {
        let mut v = Vec::new();
        if let Some(b) = BaseDirs::new() {
            let xdg = b.home_dir().join(".config").join("lgg").join("config.toml");
            v.push(xdg);
            let native = b.config_dir().join("lgg").join("config.toml");
            v.push(native);
        }
        v
    }

    /// Read the first existing config file and parse it.
    fn read_file_config() -> Result<FileConfig> {
        for path in Self::config_file_paths() {
            if !path.exists() {
                continue;
            }
            let s =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            return Self::parse_file(&s).with_context(|| format!("parsing {}", path.display()));
        }
        Ok(FileConfig {
            journal_dir: None,
            editor: None,
            default_time: None,
            synonyms: None,
            date_format: None,
        })
    }

    /// Parse a TOML string into `FileConfig`.
    fn parse_file(s: &str) -> Result<FileConfig> {
        Ok(toml::from_str::<FileConfig>(s)?)
    }

    /// Merge `[synonyms]` into the global Keywords registry.
    /// Omits synonyms that collide with current canonical Keyword (eg. "today").
    /// Lowercases both alias and target for case-insensitive behavior.
    fn load_synonyms(synonyms: &Option<HashMap<String, String>>) {
        match synonyms {
            Some(map) if !map.is_empty() => {
                let pairs: Vec<(String, String)> = map
                    .iter()
                    .filter(|(alias, _)| !Keywords::is_canonical(alias))
                    .map(|(a, t)| (a.clone(), t.clone()))
                    .collect();

                if !pairs.is_empty() {
                    Keywords::extend(&pairs);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use super::*;
    use crate::keywords::{Keyword, Keywords};
    use chrono::NaiveTime;
    use std::path::PathBuf;

    /// Test helper to create a default `Config` for testing purposes.
    ///
    /// This is the single source of truth for test configuration.
    /// If you add a field to `Config`, you only need to update it here.
    pub(crate) fn mk_config(journal_dir: PathBuf) -> Config {
        Config {
            journal_dir,
            editor: None,
            default_time: NaiveTime::from_hms_opt(21, 0, 0).expect("valid time"),
            date_format: "%A, %d %b %Y".to_string(),
        }
    }

    #[test]
    fn candidates_prioritize_xdg_then_native() {
        if let Some(b) = BaseDirs::new() {
            let expected_xdg = b.home_dir().join(".config").join("lgg").join("config.toml");
            let expected_native = b.config_dir().join("lgg").join("config.toml");
            let c = super::Config::config_file_paths();
            assert_eq!(c.get(0), Some(&expected_xdg));
            assert_eq!(c.get(1), Some(&expected_native));
        }
    }

    #[test]
    fn parse_file_accepts_journal_dir_and_editor() {
        let toml = r#"
            journal_dir = "/tmp/my-journal"
            editor = "hx"
        "#;
        let fc = super::Config::parse_file(toml).unwrap();
        assert_eq!(
            fc.journal_dir.as_deref(),
            Some(Path::new("/tmp/my-journal"))
        );
        assert_eq!(fc.editor.as_deref(), Some("hx"));
    }

    #[test]
    fn parse_file_accepts_synonyms_and_extends_registry() {
        let toml = r#"
            journal_dir = "/tmp/my-journal"

            [synonyms]
            ytd = "yesterday"
            AYER = "yesterday"
        "#;

        let fc = super::Config::parse_file(toml).unwrap();
        assert!(fc.synonyms.is_some());

        super::Config::load_synonyms(&fc.synonyms);

        assert!(Keywords::matches(Keyword::Yesterday, "ytd"));
        assert!(Keywords::matches(Keyword::Yesterday, "ayer"));
    }

    #[test]
    fn parse_file_no_accepts_canonical_synonyms() {
        let toml = r#"
            journal_dir = "/tmp/my-journal"

            [synonyms]
            today = "yesterday"
            ytd = "yesterday"
        "#;

        let fc = super::Config::parse_file(toml).unwrap();
        assert!(fc.synonyms.is_some());

        super::Config::load_synonyms(&fc.synonyms);

        assert!(!Keywords::matches(Keyword::Yesterday, "today"));
        assert!(Keywords::matches(Keyword::Yesterday, "ytd"));
    }
}
