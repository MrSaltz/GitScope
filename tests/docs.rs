use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

const PAIRS: [&str; 8] = [
    "README",
    "CONTRIBUTING",
    "CHANGELOG",
    "docs/ARCHITECTURE",
    "docs/JSON",
    "docs/MARKDOWN",
    "docs/STABILITY",
    "docs/RELEASING",
];

fn path_of(doc: &str, english: bool) -> PathBuf {
    let suffix = if english { "-EN" } else { "" };
    Path::new(ROOT).join(format!("{doc}{suffix}.md"))
}

struct Doc {
    headings: Vec<(usize, String)>,
    fences: usize,
    table_rows: usize,
    links: Vec<String>,
}

fn parse(text: &str) -> Doc {
    let mut doc = Doc {
        headings: Vec::new(),
        fences: 0,
        table_rows: 0,
        links: Vec::new(),
    };
    let mut open_fence: Option<(char, usize)> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        let first = trimmed.chars().next();
        let run = match first {
            Some(c @ ('`' | '~')) => trimmed.chars().take_while(|&x| x == c).count(),
            _ => 0,
        };

        match open_fence {
            Some((ch, len)) => {
                if indent <= 3
                    && first == Some(ch)
                    && run >= len
                    && trimmed[run..].trim().is_empty()
                {
                    open_fence = None;
                }
                continue;
            }
            None if indent <= 3
                && run >= 3
                && !(first == Some('`') && trimmed[run..].contains('`')) =>
            {
                open_fence = first.map(|c| (c, run));
                doc.fences += 1;
                continue;
            }
            None => {}
        }

        let hashes = line.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
            doc.headings
                .push((hashes, line[hashes..].trim().to_owned()));
        }
        if trimmed.starts_with('|') {
            doc.table_rows += 1;
        }
        doc.links.extend(links_in(line));
    }
    doc
}

fn links_in(line: &str) -> Vec<String> {
    let mut text = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let n = chars[i..].iter().take_while(|&&c| c == '`').count();
            let close = (i + n..chars.len()).find(|&j| {
                chars[j..].iter().take_while(|&&c| c == '`').count() == n
                    && (j == 0 || chars[j - 1] != '`')
            });
            match close {
                Some(j) => i = j + n,
                None => {
                    i += n;
                }
            }
            continue;
        }
        text.push(chars[i]);
        i += 1;
    }

    let mut links = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("](") {
        let after = &rest[at + 2..];
        let Some(end) = after.find(')') else { break };
        let target = after[..end].split_whitespace().next().unwrap_or("");
        links.push(target.to_owned());
        rest = &after[end..];
    }
    links
}

fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn anchors(doc: &Doc) -> HashSet<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut anchors = HashSet::new();
    for (_, text) in &doc.headings {
        let base = slug(text);
        let n = seen.entry(base.clone()).or_default();
        anchors.insert(if *n == 0 { base } else { format!("{base}-{n}") });
        *n += 1;
    }
    anchors
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn all_docs() -> Vec<PathBuf> {
    PAIRS
        .iter()
        .flat_map(|doc| [path_of(doc, false), path_of(doc, true)])
        .collect()
}

fn is_external(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://") || target.starts_with("mailto:")
}

#[test]
fn every_markdown_file_has_a_twin_in_the_other_language() {
    let mut found = Vec::new();
    for dir in [Path::new(ROOT).to_path_buf(), Path::new(ROOT).join("docs")] {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "md") {
                found.push(path);
            }
        }
    }
    let expected: HashSet<PathBuf> = all_docs().into_iter().collect();
    for path in &found {
        assert!(
            expected.contains(path),
            "{} has no twin: add it to PAIRS and translate it",
            path.display()
        );
    }
    for path in &expected {
        assert!(path.exists(), "{} is missing", path.display());
    }
}

#[test]
fn links_and_anchors_resolve() {
    let mut problems = Vec::new();
    for path in all_docs() {
        let doc = parse(&read(&path));
        let dir = path.parent().unwrap();
        for target in &doc.links {
            if is_external(target) {
                continue;
            }
            let (file_part, fragment) = target.split_once('#').unwrap_or((target.as_str(), ""));
            let file = if file_part.is_empty() {
                path.clone()
            } else {
                dir.join(file_part)
            };
            if !file.exists() {
                problems.push(format!(
                    "{}: `{target}` points to a missing file",
                    path.display()
                ));
                continue;
            }
            if !fragment.is_empty() {
                if file.extension().is_none_or(|e| e != "md") {
                    problems.push(format!(
                        "{}: `{target}` has an anchor on a non-Markdown file",
                        path.display()
                    ));
                    continue;
                }
                if !anchors(&parse(&read(&file))).contains(fragment) {
                    problems.push(format!(
                        "{}: `{target}` has no matching heading",
                        path.display()
                    ));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "broken links:\n{}",
        problems.join("\n")
    );
}

#[test]
fn both_languages_have_the_same_structure() {
    for doc in PAIRS {
        let pt = parse(&read(&path_of(doc, false)));
        let en = parse(&read(&path_of(doc, true)));

        let levels = |d: &Doc| {
            d.headings
                .iter()
                .map(|(level, _)| *level)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            levels(&pt),
            levels(&en),
            "{doc}: the headings (count and levels) differ between the languages"
        );
        assert_eq!(pt.fences, en.fences, "{doc}: number of code blocks differs");
        assert_eq!(
            pt.table_rows, en.table_rows,
            "{doc}: number of table rows differs"
        );
        assert_eq!(
            pt.links.len(),
            en.links.len(),
            "{doc}: number of links differs"
        );
        let external = |d: &Doc| d.links.iter().filter(|l| is_external(l)).count();
        assert_eq!(
            external(&pt),
            external(&en),
            "{doc}: number of external links differs"
        );
    }
}

#[test]
fn each_language_links_to_its_own_language() {
    for doc in PAIRS {
        for english in [false, true] {
            let path = path_of(doc, english);
            let parsed = parse(&read(&path));
            let crossing: Vec<&String> = parsed
                .links
                .iter()
                .filter(|target| !is_external(target))
                .filter(|target| {
                    let file = target.split('#').next().unwrap_or("");
                    file.ends_with(".md") && file.ends_with("-EN.md") != english
                })
                .collect();
            assert_eq!(
                crossing.len(),
                1,
                "{}: only the language switcher may link to the other language, found {crossing:?}",
                path.display()
            );
            assert!(
                parsed
                    .links
                    .first()
                    .is_some_and(|first| first == crossing[0]),
                "{}: the language switcher must be the first link",
                path.display()
            );
        }
    }
}

#[test]
fn the_slug_matches_what_github_generates() {
    assert_eq!(slug("Repositórios remotos"), "repositórios-remotos");
    assert_eq!(slug("Saída JSON"), "saída-json");
    assert_eq!(
        slug("How the numbers are defined"),
        "how-the-numbers-are-defined"
    );
    assert_eq!(slug("`update` vs. `markdown`"), "update-vs-markdown");
    assert_eq!(slug("Exit codes and streams"), "exit-codes-and-streams");
}

#[test]
fn the_parser_ignores_code_blocks_and_inline_code() {
    let text = "# Título\n\n```markdown\n# não é título\n[x](nada.md)\n```\n\nVeja [a](b.md#c) e `[d](e.md)`.\n\n| a | b |\n";
    let doc = parse(text);
    assert_eq!(doc.headings, vec![(1, "Título".to_owned())]);
    assert_eq!(doc.fences, 1);
    assert_eq!(doc.links, vec!["b.md#c".to_owned()]);
    assert_eq!(doc.table_rows, 1);
}
