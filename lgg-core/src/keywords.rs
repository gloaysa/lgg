use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, AsRefStr, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Keyword {
    At,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    Today,
    Yesterday,
    Tomorrow,
    Morning,
    Noon,
    Evening,
    Night,
    Midnight,
    #[strum(serialize = "last week")]
    LastWeek,
    LastMonth,
    LastYear,
    ThisWeek,
    ThisMonth,
    ThisYear,
}

pub struct Keywords;

impl Keywords {
    /// Returns the **global keyword registry** (input → canonical).
    ///
    /// The registry is:
    /// - **Initialized once** on first access (via [`once_cell::sync::Lazy`]).
    /// - **Thread-safe** (wrapped in [`RwLock`]): many readers or one writer.
    /// - **Lowercased**: all keys are stored lowercased for case-insensitive lookups.
    ///
    /// Seeded entries (canonical → canonical):
    /// - `"today"` → `"today"`
    /// - `"yesterday"` → `"yesterday"`
    /// - `"tomorrow"` → `"tomorrow"`
    /// - `"last week"` → `"last week"`
    ///
    /// You normally **don’t call this directly**; use [`extend`](Self::extend)
    /// to add synonyms and [`matches`](Self::matches) for checks.
    ///
    /// References:
    /// - once_cell::sync::Lazy: <https://docs.rs/once_cell>
    /// - std::sync::RwLock: <https://doc.rust-lang.org/std/sync/struct.RwLock.html>
    fn registry() -> &'static RwLock<HashMap<String, Keyword>> {
        static REGISTRY: Lazy<RwLock<HashMap<String, Keyword>>> = Lazy::new(|| {
            let mut m = HashMap::new();
            m.insert("at".to_string(), Keyword::At);
            m.insert("monday".to_string(), Keyword::Monday);
            m.insert("tuesday".to_string(), Keyword::Tuesday);
            m.insert("wednesday".to_string(), Keyword::Wednesday);
            m.insert("thursday".to_string(), Keyword::Thursday);
            m.insert("friday".to_string(), Keyword::Friday);
            m.insert("saturday".to_string(), Keyword::Saturday);
            m.insert("sunday".to_string(), Keyword::Sunday);
            m.insert("today".to_string(), Keyword::Today);
            m.insert("yesterday".to_string(), Keyword::Yesterday);
            m.insert("tomorrow".to_string(), Keyword::Tomorrow);
            m.insert("morning".to_string(), Keyword::Morning);
            m.insert("noon".to_string(), Keyword::Noon);
            m.insert("evening".to_string(), Keyword::Evening);
            m.insert("night".to_string(), Keyword::Night);
            m.insert("midnight".to_string(), Keyword::Midnight);
            m.insert("last week".to_string(), Keyword::LastWeek);
            m.insert("last month".to_string(), Keyword::LastMonth);
            m.insert("last year".to_string(), Keyword::LastYear);
            m.insert("this week".to_string(), Keyword::ThisWeek);
            m.insert("this month".to_string(), Keyword::ThisMonth);
            m.insert("this year".to_string(), Keyword::ThisYear);

            RwLock::new(m)
        });
        &REGISTRY
    }

    /// Extends the global registry with user-defined **synonyms**.
    ///
    /// Each pair is `(alias, target)`. The `target` must be a **known** keyword already
    /// in the registry (typically a canonical constant or an existing synonym that maps
    /// to a canonical). If `target` isn’t known, the pair is ignored silently.
    ///
    /// All keys are normalized to **lowercase** to keep lookups case-insensitive.
    ///
    /// Typical call site: during `Config::load()`, after reading `[synonyms]`
    /// from `config.toml`:
    ///
    /// ```toml
    /// // config.toml
    /// // [synonyms]
    /// // ytd  = "yesterday"
    /// // ayer = "yesterday"
    /// // tmrw = "tomorrow"
    ///
    /// let pairs: Vec<(String, String)> = cfg.synonyms.iter()
    ///     .map(|(alias, target)| (alias.clone(), target.clone()))
    ///     .collect();
    /// Keywords::extend(&pairs);
    /// ```
    pub fn extend(synonyms: &[(String, String)]) {
        let mut reg = Self::registry().write().unwrap();
        for (alias, target) in synonyms {
            if let Some(&canonical) = reg.get(&target.to_ascii_lowercase()) {
                reg.insert(alias.to_ascii_lowercase(), canonical);
            }
        }
    }

    /// Returns `true` if `word` is a canonical word (eg "today").
    pub fn is_canonical(word: &str) -> bool {
        Keyword::iter().any(|key| key.as_ref() == word)
    }

    /// Returns `true` if `input` equals (case-insensitively) the given **canonical keyword**
    /// or any of its registered synonyms.
    ///
    /// Example:
    /// ```rs
    /// use crate::keywords::{Keywords, YESTERDAY};
    ///
    /// assert!(Keywords::matches(YESTERDAY, "yesterday"));
    /// assert!(Keywords::matches(YESTERDAY, "YESTERDAY"));
    /// assert!(!Keywords::matches(YESTERDAY, "today"));
    /// ```
    pub fn matches(keyword: Keyword, input: &str) -> bool {
        let reg = Self::registry().read().unwrap();
        reg.get(&input.to_ascii_lowercase())
            .map(|&canon| canon == keyword)
            .unwrap_or(false)
    }

    pub fn find_word(keyword: Keyword, input: &str) -> Option<String> {
        let lower = input.to_ascii_lowercase();
        let pattern = format!(r"\b{}\b", regex::escape(keyword.as_ref()));
        let re = Regex::new(&pattern).unwrap();
        if re.is_match(&lower) {
            Some(keyword.as_ref().to_string())
        } else {
            None
        }
    }

    pub fn find_position(keyword: Keyword, input: &str) -> Option<usize> {
        let lower = input.to_ascii_lowercase();
        let pattern = format!(r"\b{}\b", regex::escape(keyword.as_ref()));
        let re = Regex::new(&pattern).unwrap();
        if let Some(m) = re.find(&lower) {
            Some(m.start())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_constants() {
        assert!(Keywords::matches(Keyword::Today, "today"));
        assert!(Keywords::matches(Keyword::Yesterday, "yesterday"));
    }

    #[test]
    fn synonyms_extend() {
        Keywords::extend(&[
            ("ytd".into(), "yesterday".into()),
            ("ayer".into(), "yesterday".into()),
            ("tmrw".into(), "tomorrow".into()),
        ]);
        assert!(Keywords::matches(Keyword::Yesterday, "ytd"));
        assert!(Keywords::matches(Keyword::Yesterday, "ayer"));
        assert!(Keywords::matches(Keyword::Tomorrow, "tmrw"));
    }

    #[test]
    fn unknown_word_in_matches_returns_none() {
        if Keywords::matches(Keyword::Tomorrow, "not in registry") {
            assert!(false);
        } else {
            assert!(true);
        }
    }

    #[test]
    fn find_word_finds_whole_word() {
        assert_eq!(Keywords::find_word(Keyword::At, "text at text"), Some("at".to_string()));
        assert_eq!(Keywords::find_word(Keyword::Friday, "go friday go"), Some("friday".to_string()));
    }

    #[test]
    fn find_word_does_not_find_substring() {
        assert_eq!(Keywords::find_word(Keyword::At, "saturday"), None);
        assert_eq!(Keywords::find_word(Keyword::Friday, "fridaya"), None);
    }

    #[test]
    fn find_position_finds_whole_word() {
        assert_eq!(Keywords::find_position(Keyword::At, "text at text"), Some(5));
        assert_eq!(Keywords::find_position(Keyword::Friday, "go friday go"), Some(3));
    }

    #[test]
    fn find_position_does_not_find_substring() {
        assert_eq!(Keywords::find_position(Keyword::At, "saturday"), None);
        assert_eq!(Keywords::find_position(Keyword::Friday, "fridaya"), None);
    }

    #[test]
    fn unknown_word_in_find_returns_none() {
        if let Some(_) = Keywords::find_position(Keyword::Tomorrow, "text text text") {
            assert!(false);
        } else {
            assert!(true);
        }
    }

}
