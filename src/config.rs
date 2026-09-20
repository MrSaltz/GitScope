use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::output::markdown::write_atomic;

pub const CONFIG_ENV: &str = "GITSCOPE_CONFIG";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en", alias = "en-US", alias = "en_US")]
    English,
    #[serde(rename = "pt", alias = "pt-BR", alias = "pt_BR")]
    Portuguese,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::Portuguese];

    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Portuguese => "Português (Brasil)",
        }
    }

    pub fn next(self) -> Language {
        let at = Language::ALL.iter().position(|&l| l == self).unwrap_or(0);
        Language::ALL[(at + 1) % Language::ALL.len()]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub language: Language,
}

impl Config {
    pub fn load(path: &Path) -> std::result::Result<Config, String> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
        };
        serde_json::from_str(&text)
            .map_err(|err| format!("invalid configuration file {}: {err}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let mut text = serde_json::to_string_pretty(self).expect("a configuração sempre serializa");
        text.push('\n');
        write_atomic(path, &text)
    }
}

pub fn default_path() -> Option<PathBuf> {
    path_with(
        |name| env::var_os(name),
        cfg!(windows),
        cfg!(target_os = "macos"),
    )
}

fn path_with(
    var: impl Fn(&str) -> Option<OsString>,
    windows: bool,
    macos: bool,
) -> Option<PathBuf> {
    let non_empty = |name: &str| var(name).filter(|v| !v.is_empty()).map(PathBuf::from);

    if let Some(explicit) = non_empty(CONFIG_ENV) {
        return Some(explicit);
    }
    let base = if windows {
        non_empty("APPDATA")?
    } else if macos {
        non_empty("HOME")?
            .join("Library")
            .join("Application Support")
    } else {
        non_empty("XDG_CONFIG_HOME").or_else(|| non_empty("HOME").map(|h| h.join(".config")))?
    };
    Some(base.join("gitscope").join("config.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    #[test]
    fn the_default_is_english() {
        assert_eq!(Config::default().language, Language::English);
    }

    #[test]
    fn languages_cycle_and_have_native_names() {
        assert_eq!(Language::English.next(), Language::Portuguese);
        assert_eq!(Language::Portuguese.next(), Language::English);
        assert_eq!(Language::English.native_name(), "English");
        assert_eq!(Language::Portuguese.native_name(), "Português (Brasil)");
    }

    #[test]
    fn a_missing_file_is_the_default_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(&dir.path().join("nope.json")).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn saving_creates_the_directories_and_loading_reads_it_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("config.json");
        let config = Config {
            language: Language::Portuguese,
        };

        config.save(&path).unwrap();

        assert_eq!(Config::load(&path).unwrap(), config);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{\n  \"language\": \"pt\"\n}\n"
        );
        Config::default().save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap().language, Language::English);
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn accepted_spellings_unknown_keys_and_empty_objects() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        for (text, expected) in [
            (r#"{"language": "pt"}"#, Language::Portuguese),
            (r#"{"language": "pt-BR"}"#, Language::Portuguese),
            (r#"{"language": "pt_BR"}"#, Language::Portuguese),
            (r#"{"language": "en"}"#, Language::English),
            (r#"{}"#, Language::English),
            (
                r#"{"language": "pt", "future_option": [1, 2]}"#,
                Language::Portuguese,
            ),
        ] {
            fs::write(&path, text).unwrap();
            assert_eq!(Config::load(&path).unwrap().language, expected, "{text}");
        }
    }

    #[test]
    fn a_broken_file_is_reported_with_its_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        for text in ["not json", r#"{"language": "klingon"}"#, "42"] {
            fs::write(&path, text).unwrap();
            let message = Config::load(&path).unwrap_err();
            assert!(message.contains("invalid configuration file"), "{message}");
            assert!(message.contains("config.json"), "{message}");
        }
    }

    #[test]
    fn the_path_follows_the_conventions_of_each_system() {
        let linux = path_with(env_of(&[("HOME", "/home/me")]), false, false);
        assert_eq!(
            linux,
            Some(PathBuf::from("/home/me/.config/gitscope/config.json"))
        );

        let xdg = path_with(
            env_of(&[("HOME", "/home/me"), ("XDG_CONFIG_HOME", "/cfg")]),
            false,
            false,
        );
        assert_eq!(xdg, Some(PathBuf::from("/cfg/gitscope/config.json")));

        let mac = path_with(env_of(&[("HOME", "/Users/me")]), false, true);
        assert_eq!(
            mac,
            Some(PathBuf::from(
                "/Users/me/Library/Application Support/gitscope/config.json"
            ))
        );

        let windows = path_with(
            env_of(&[("APPDATA", "C:/Users/me/AppData/Roaming")]),
            true,
            false,
        );
        assert_eq!(
            windows,
            Some(PathBuf::from(
                "C:/Users/me/AppData/Roaming/gitscope/config.json"
            ))
        );
    }

    #[test]
    fn the_environment_variable_wins_and_a_missing_home_means_no_path() {
        let explicit = path_with(
            env_of(&[(CONFIG_ENV, "/tmp/mine.json"), ("HOME", "/home/me")]),
            false,
            false,
        );
        assert_eq!(explicit, Some(PathBuf::from("/tmp/mine.json")));

        assert_eq!(path_with(env_of(&[]), false, false), None);
        assert_eq!(path_with(env_of(&[]), true, false), None);
        assert_eq!(path_with(env_of(&[("HOME", "")]), false, false), None);
        assert_eq!(
            path_with(env_of(&[(CONFIG_ENV, ""), ("HOME", "/h")]), false, false),
            Some(PathBuf::from("/h/.config/gitscope/config.json"))
        );
    }
}
