use std::collections::HashMap;

use super::files::extension_of;
use super::percentage;
use crate::model::LanguageStats;

pub const LANGUAGES: &[(&str, &str)] = &[
    ("rs", "Rust"),
    ("ts", "TypeScript"),
    ("tsx", "TypeScript"),
    ("js", "JavaScript"),
    ("jsx", "JavaScript"),
    ("mjs", "JavaScript"),
    ("cjs", "JavaScript"),
    ("py", "Python"),
    ("go", "Go"),
    ("java", "Java"),
    ("kt", "Kotlin"),
    ("kts", "Kotlin"),
    ("swift", "Swift"),
    ("c", "C"),
    ("h", "C"),
    ("cpp", "C++"),
    ("cc", "C++"),
    ("cxx", "C++"),
    ("hpp", "C++"),
    ("hh", "C++"),
    ("cs", "C#"),
    ("rb", "Ruby"),
    ("php", "PHP"),
    ("sh", "Shell"),
    ("bash", "Shell"),
    ("zsh", "Shell"),
    ("ps1", "PowerShell"),
    ("lua", "Lua"),
    ("dart", "Dart"),
    ("scala", "Scala"),
    ("hs", "Haskell"),
    ("ex", "Elixir"),
    ("exs", "Elixir"),
    ("erl", "Erlang"),
    ("clj", "Clojure"),
    ("r", "R"),
    ("sql", "SQL"),
    ("html", "HTML"),
    ("css", "CSS"),
    ("scss", "SCSS"),
    ("vue", "Vue"),
    ("svelte", "Svelte"),
    ("zig", "Zig"),
    ("nim", "Nim"),
];

pub fn language_for(extension: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|(ext, _)| *ext == extension)
        .map(|(_, language)| *language)
}

pub fn language_stats(snapshot: &[String]) -> Vec<LanguageStats> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for path in snapshot {
        if let Some(language) = extension_of(path).as_deref().and_then(language_for) {
            *counts.entry(language).or_default() += 1;
        }
    }
    let total: usize = counts.values().sum();

    let mut stats: Vec<LanguageStats> = counts
        .into_iter()
        .map(|(name, files)| LanguageStats {
            name: name.to_owned(),
            files,
            percentage: percentage(files, total),
        })
        .collect();
    stats.sort_by(|a, b| b.files.cmp(&a.files).then_with(|| a.name.cmp(&b.name)));
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| (*p).to_owned()).collect()
    }

    #[test]
    fn extensions_map_to_languages() {
        assert_eq!(language_for("rs"), Some("Rust"));
        assert_eq!(language_for("tsx"), Some("TypeScript"));
        assert_eq!(language_for("cpp"), Some("C++"));
        assert_eq!(language_for("md"), None);
    }

    #[test]
    fn table_has_no_duplicate_extensions() {
        let mut seen = std::collections::HashSet::new();
        for (ext, _) in LANGUAGES {
            assert!(seen.insert(*ext), "duplicate extension {ext}");
            assert_eq!(*ext, ext.to_lowercase());
        }
    }

    #[test]
    fn several_extensions_can_share_one_language() {
        let stats = language_stats(&snapshot(&["a.ts", "b.tsx", "c.py"]));
        assert_eq!(stats[0].name, "TypeScript");
        assert_eq!(stats[0].files, 2);
        assert_eq!(stats[0].percentage, 66.67);
        assert_eq!(stats[1].name, "Python");
        assert_eq!(stats[1].percentage, 33.33);
    }

    #[test]
    fn unknown_and_missing_extensions_are_ignored() {
        let stats = language_stats(&snapshot(&["a.rs", "notes.md", "LICENSE", "data.json"]));
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "Rust");
        assert_eq!(stats[0].percentage, 100.0);
    }

    #[test]
    fn matching_is_case_insensitive() {
        let stats = language_stats(&snapshot(&["MAIN.RS"]));
        assert_eq!(stats[0].name, "Rust");
    }

    #[test]
    fn nothing_recognised_gives_an_empty_list() {
        assert!(language_stats(&snapshot(&["README.md"])).is_empty());
        assert!(language_stats(&[]).is_empty());
    }
}
