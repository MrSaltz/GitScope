use super::format::{day, thousands};
use crate::model::RepositoryStats;

pub const NOT_AVAILABLE: &str = "n/a";

#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    pub stats: &'a RepositoryStats,
}

impl<'a> Context<'a> {
    pub fn new(stats: &'a RepositoryStats) -> Self {
        Self { stats }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    Unknown,
    Arguments(String),
}

pub type Resolver = fn(&Context<'_>, &[&str]) -> Result<String, ResolveError>;

#[derive(Clone, Copy)]
pub struct VariableDef {
    pub name: &'static str,
    pub group: &'static str,
    pub arguments: &'static str,
    pub description: &'static str,
    pub resolve: Resolver,
}

impl VariableDef {
    pub fn usage(&self) -> String {
        if self.arguments.is_empty() {
            format!("{{{}}}", self.name)
        } else {
            format!("{{{}:{}}}", self.name, self.arguments)
        }
    }
}

const fn plain(
    group: &'static str,
    name: &'static str,
    description: &'static str,
    resolve: Resolver,
) -> VariableDef {
    VariableDef {
        name,
        group,
        arguments: "",
        description,
        resolve,
    }
}

#[derive(Clone)]
pub struct VariableRegistry {
    defs: Vec<VariableDef>,
}

impl VariableRegistry {
    pub fn empty() -> Self {
        Self { defs: Vec::new() }
    }

    pub fn register(&mut self, def: VariableDef) {
        match self.defs.iter_mut().find(|d| d.name == def.name) {
            Some(existing) => *existing = def,
            None => self.defs.push(def),
        }
    }

    pub fn standard() -> Self {
        let mut registry = Self::empty();
        for def in standard_variables() {
            registry.register(def);
        }
        registry
    }

    pub fn get(&self, name: &str) -> Option<&VariableDef> {
        self.defs.iter().find(|d| d.name == name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &VariableDef> {
        self.defs.iter()
    }

    pub fn describe(&self) -> String {
        const USAGE_WIDTH: usize = 26;
        let mut groups: Vec<&str> = Vec::new();
        for def in &self.defs {
            if !groups.contains(&def.group) {
                groups.push(def.group);
            }
        }

        let mut text = String::new();
        for group in groups {
            text.push_str(group);
            text.push('\n');
            for def in self.defs.iter().filter(|d| d.group == group) {
                let usage = def.usage();
                if usage.len() <= USAGE_WIDTH {
                    text.push_str(&format!("  {usage:<USAGE_WIDTH$}  {}\n", def.description));
                } else {
                    text.push_str(&format!(
                        "  {usage}\n  {:<USAGE_WIDTH$}  {}\n",
                        "", def.description
                    ));
                }
            }
            text.push('\n');
        }
        text.push_str(
            "Write a variable as {name} in a Markdown file; escape it as \\{name} to keep it literal.\n",
        );
        text
    }

    pub fn names(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.defs.iter().map(|d| d.name).collect();
        names.sort_unstable();
        names
    }

    pub fn resolve(
        &self,
        context: &Context<'_>,
        name: &str,
        args: &[&str],
    ) -> Result<String, ResolveError> {
        let def = self.get(name).ok_or(ResolveError::Unknown)?;
        if def.arguments.is_empty() && !args.is_empty() {
            return Err(ResolveError::Arguments(format!(
                "{} takes no arguments",
                def.usage()
            )));
        }
        (def.resolve)(context, args)
    }

    pub fn suggest(&self, name: &str) -> Option<&'static str> {
        let wanted = name.to_lowercase();
        self.defs
            .iter()
            .map(|d| (edit_distance(&wanted, d.name), d.name))
            .filter(|(distance, _)| *distance <= 2)
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, candidate)| candidate)
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = if ca == *cb {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[j])
            };
            diagonal = above;
        }
    }
    row[b.len()]
}

fn or_na(value: Option<String>) -> String {
    value.unwrap_or_else(|| NOT_AVAILABLE.to_owned())
}

fn standard_variables() -> Vec<VariableDef> {
    vec![
        plain(
            "Repository",
            "repository_name",
            "Name of the repository",
            |c, _| Ok(c.stats.repository.name.clone()),
        ),
        plain(
            "Repository",
            "first_commit",
            "Date of the first commit (YYYY-MM-DD)",
            |c, _| Ok(or_na(c.stats.commits.first.map(day))),
        ),
        plain(
            "Repository",
            "last_commit",
            "Date of the last commit (YYYY-MM-DD)",
            |c, _| Ok(or_na(c.stats.commits.last.map(day))),
        ),
        plain(
            "Repository",
            "period",
            "First and last commit dates, e.g. 2026-01-03 → 2026-09-18",
            |c, _| {
                Ok(or_na(c.stats.commits.first.zip(c.stats.commits.last).map(
                    |(first, last)| format!("{} → {}", day(first), day(last)),
                )))
            },
        ),
        plain("Repository", "branches", "Number of branches", |c, _| {
            Ok(thousands(c.stats.repository.branches))
        }),
        plain("Repository", "tags", "Number of tags", |c, _| {
            Ok(thousands(c.stats.repository.tags))
        }),
        plain("Commits", "commits", "Number of commits", |c, _| {
            Ok(thousands(c.stats.commits.total))
        }),
        plain(
            "Commits",
            "first_commit_message",
            "First line of the first commit's message",
            |c, _| Ok(or_na(c.stats.commits.first_message.clone())),
        ),
        plain(
            "Commits",
            "last_commit_message",
            "First line of the last commit's message",
            |c, _| Ok(or_na(c.stats.commits.last_message.clone())),
        ),
        plain(
            "Commits",
            "most_active_hour",
            "Hour of the day (UTC) with most commits, e.g. 14:00",
            |c, _| {
                Ok(or_na(
                    c.stats
                        .commits
                        .most_active_hour
                        .map(|h| format!("{h:02}:00")),
                ))
            },
        ),
        plain(
            "Contributors",
            "contributors",
            "Number of contributors",
            |c, _| Ok(thousands(c.stats.contributors.len())),
        ),
        plain(
            "Contributors",
            "top_author",
            "Name of the contributor with most commits",
            |c, _| Ok(or_na(c.stats.contributors.first().map(|a| a.name.clone()))),
        ),
        plain(
            "Contributors",
            "top_author_commits",
            "Commits of the top contributor",
            |c, _| {
                Ok(or_na(
                    c.stats.contributors.first().map(|a| thousands(a.commits)),
                ))
            },
        ),
        plain(
            "Contributors",
            "top_author_percentage",
            "Share of the top contributor, e.g. 54.9 (no % sign)",
            |c, _| {
                Ok(or_na(
                    c.stats
                        .contributors
                        .first()
                        .map(|a| format!("{:.1}", a.percentage)),
                ))
            },
        ),
        VariableDef {
            name: "author",
            group: "Contributors",
            arguments: "NAME:commits|insertions|deletions",
            description: "One contributor (matched by exact name or email, case-insensitive)",
            resolve: resolve_author,
        },
        plain(
            "Files",
            "files",
            "Number of files in the current snapshot",
            |c, _| Ok(thousands(c.stats.files.total)),
        ),
        plain("Files", "insertions", "Total lines added", |c, _| {
            Ok(thousands(c.stats.commits.insertions))
        }),
        plain("Files", "deletions", "Total lines removed", |c, _| {
            Ok(thousands(c.stats.commits.deletions))
        }),
        plain(
            "Languages",
            "language_count",
            "Number of recognised languages",
            |c, _| Ok(thousands(c.stats.languages.len())),
        ),
        plain(
            "Languages",
            "top_language",
            "Language with most files",
            |c, _| Ok(or_na(c.stats.languages.first().map(|l| l.name.clone()))),
        ),
    ]
}

fn resolve_author(context: &Context<'_>, args: &[&str]) -> Result<String, ResolveError> {
    let [name, field] = args else {
        return Err(ResolveError::Arguments(
            "expected {author:NAME:FIELD}, for example {author:Alice:commits}".to_owned(),
        ));
    };
    let name = name.trim();
    let field = field.trim();

    let matching: Vec<_> = context
        .stats
        .contributors
        .iter()
        .filter(|c| c.name.eq_ignore_ascii_case(name) || c.email.eq_ignore_ascii_case(name))
        .collect();
    if matching.is_empty() {
        return Err(ResolveError::Arguments(format!(
            "no contributor named '{name}' in the analysed history"
        )));
    }

    let sum = |value: fn(&crate::model::ContributorStats) -> usize| -> usize {
        matching.iter().map(|c| value(c)).sum()
    };
    let total = match field {
        "commits" => sum(|c| c.commits),
        "insertions" => sum(|c| c.insertions),
        "deletions" => sum(|c| c.deletions),
        other => {
            return Err(ResolveError::Arguments(format!(
                "unknown field '{other}' for author; use commits, insertions or deletions"
            )));
        }
    };
    Ok(thousands(total))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::analysis::{self, testutil::record_with};
    use crate::model::{Filters, RepositoryInfo};

    pub(crate) fn fixture() -> RepositoryStats {
        let mut first = record_with(
            "Alice",
            "alice@x.io",
            "2026-01-05 14:10",
            &["a.rs", "b.ts"],
            10,
            1,
        );
        first.message = "feat: first".into();
        let second = record_with("Bob", "bob@x.io", "2026-02-10 09:00", &["a.rs"], 2, 0);
        let mut third = record_with(
            "Alice",
            "alice@x.io",
            "2026-03-07 14:50",
            &["a.rs"],
            1_000,
            5,
        );
        third.message = "fix: last".into();
        analysis::analyze(
            RepositoryInfo {
                name: "demo".into(),
                branches: 3,
                tags: 1,
            },
            Filters::default(),
            [first, second, third].into_iter().map(Ok),
            ["a.rs", "b.ts", "README.md"].map(String::from),
            analysis::Options::default(),
        )
        .unwrap()
    }

    pub(crate) fn empty_fixture() -> RepositoryStats {
        analysis::analyze(
            RepositoryInfo {
                name: "empty".into(),
                branches: 0,
                tags: 0,
            },
            Filters::default(),
            std::iter::empty(),
            std::iter::empty(),
            analysis::Options::default(),
        )
        .unwrap()
    }

    fn value(stats: &RepositoryStats, name: &str, args: &[&str]) -> Result<String, ResolveError> {
        VariableRegistry::standard().resolve(&Context::new(stats), name, args)
    }

    #[test]
    fn every_initial_variable_resolves_from_real_statistics() {
        let stats = fixture();
        let expected = [
            ("repository_name", "demo"),
            ("first_commit", "2026-01-05"),
            ("last_commit", "2026-03-07"),
            ("period", "2026-01-05 → 2026-03-07"),
            ("branches", "3"),
            ("tags", "1"),
            ("commits", "3"),
            ("first_commit_message", "feat: first"),
            ("last_commit_message", "fix: last"),
            ("most_active_hour", "14:00"),
            ("contributors", "2"),
            ("top_author", "Alice"),
            ("top_author_commits", "2"),
            ("top_author_percentage", "66.7"),
            ("files", "3"),
            ("insertions", "1,012"),
            ("deletions", "6"),
            ("language_count", "2"),
            ("top_language", "Rust"),
        ];
        for (name, want) in expected {
            assert_eq!(value(&stats, name, &[]).as_deref(), Ok(want), "{{{name}}}");
        }
    }

    #[test]
    fn missing_values_are_not_available_and_counts_are_zero() {
        let stats = empty_fixture();
        for name in [
            "first_commit",
            "last_commit",
            "period",
            "first_commit_message",
            "last_commit_message",
            "most_active_hour",
            "top_author",
            "top_author_commits",
            "top_author_percentage",
            "top_language",
        ] {
            assert_eq!(
                value(&stats, name, &[]).as_deref(),
                Ok(NOT_AVAILABLE),
                "{{{name}}}"
            );
        }
        for name in [
            "commits",
            "contributors",
            "files",
            "insertions",
            "deletions",
            "language_count",
            "branches",
            "tags",
        ] {
            assert_eq!(value(&stats, name, &[]).as_deref(), Ok("0"), "{{{name}}}");
        }
        assert_eq!(
            value(&stats, "repository_name", &[]).as_deref(),
            Ok("empty")
        );
    }

    #[test]
    fn compound_author_variables() {
        let stats = fixture();
        assert_eq!(
            value(&stats, "author", &["Alice", "commits"]).as_deref(),
            Ok("2")
        );
        assert_eq!(
            value(&stats, "author", &["alice", "insertions"]).as_deref(),
            Ok("1,010")
        );
        assert_eq!(
            value(&stats, "author", &["Alice", "deletions"]).as_deref(),
            Ok("6")
        );
        assert_eq!(
            value(&stats, "author", &["bob@x.io", " commits "]).as_deref(),
            Ok("1")
        );
    }

    #[test]
    fn compound_variables_report_what_is_wrong() {
        let stats = fixture();
        let message = |args: &[&str]| match value(&stats, "author", args) {
            Err(ResolveError::Arguments(m)) => m,
            other => panic!("expected an argument error, got {other:?}"),
        };
        assert!(message(&["Nobody", "commits"]).contains("no contributor named 'Nobody'"));
        assert!(message(&["Alice", "stars"]).contains("unknown field 'stars'"));
        assert!(message(&["Alice"]).contains("expected {author:NAME:FIELD}"));
        assert!(message(&[]).contains("expected {author:NAME:FIELD}"));
    }

    #[test]
    fn plain_variables_reject_arguments() {
        let stats = fixture();
        match value(&stats, "commits", &["extra"]) {
            Err(ResolveError::Arguments(m)) => assert_eq!(m, "{commits} takes no arguments"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unknown_names_are_reported() {
        assert_eq!(value(&fixture(), "nope", &[]), Err(ResolveError::Unknown));
    }

    #[test]
    fn typos_get_a_suggestion() {
        let registry = VariableRegistry::standard();
        assert_eq!(registry.suggest("comits"), Some("commits"));
        assert_eq!(registry.suggest("Commits"), Some("commits"));
        assert_eq!(registry.suggest("first_comit"), Some("first_commit"));
        assert_eq!(registry.suggest("something_that_does_not_exist"), None);
    }

    #[test]
    fn the_listing_shows_every_variable_under_its_group() {
        let registry = VariableRegistry::standard();
        let text = registry.describe();
        for group in [
            "Repository",
            "Commits",
            "Contributors",
            "Files",
            "Languages",
        ] {
            let heading = format!("{group}\n");
            assert!(text.starts_with(&heading) || text.contains(&format!("\n{heading}")));
        }
        for def in registry.iter() {
            assert!(text.contains(&def.usage()), "{}", def.name);
            assert!(text.contains(def.description), "{}", def.name);
        }
        assert!(
            text.contains("  {commits}                   Number of commits\n"),
            "{text}"
        );
        assert!(text.contains("{author:NAME:commits|insertions|deletions}\n"));
        assert!(text.ends_with("keep it literal.\n"));
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", "abd"), 1);
        assert_eq!(edit_distance("abc", "ab"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn registry_is_consistent() {
        let registry = VariableRegistry::standard();
        let mut seen = std::collections::HashSet::new();
        for def in registry.iter() {
            assert!(seen.insert(def.name), "duplicate variable {}", def.name);
            assert!(
                !def.description.is_empty(),
                "{} needs a description",
                def.name
            );
            assert!(
                def.name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{} is not snake_case",
                def.name
            );
        }
        assert!(
            registry.names().windows(2).all(|w| w[0] < w[1]),
            "names() is sorted"
        );
        assert_eq!(
            registry.get("author").unwrap().usage(),
            "{author:NAME:commits|insertions|deletions}"
        );
        assert_eq!(registry.get("commits").unwrap().usage(), "{commits}");
    }

    #[test]
    fn registering_extends_and_overrides() {
        let mut registry = VariableRegistry::standard();
        registry.register(plain("Custom", "answer", "The answer", |_, _| {
            Ok("42".into())
        }));
        registry.register(plain("Custom", "commits", "Overridden", |_, _| {
            Ok("many".into())
        }));

        let stats = fixture();
        let context = Context::new(&stats);
        assert_eq!(
            registry.resolve(&context, "answer", &[]).as_deref(),
            Ok("42")
        );
        assert_eq!(
            registry.resolve(&context, "commits", &[]).as_deref(),
            Ok("many")
        );
        assert_eq!(registry.iter().filter(|d| d.name == "commits").count(), 1);
    }
}
