//! Benchmarks: `cargo bench` (release). Variáveis: `GITSCOPE_BENCH_COMMITS` (1000) e
//! `GITSCOPE_BENCH_RUNS` (7). Os números servem para comparar versões, não como promessa.

#[path = "../tests/common/mod.rs"]
mod common;

use std::env;
use std::hint::black_box;
use std::time::{Duration, Instant};

use common::TestRepo;
use gitscope::analysis::Options;
use gitscope::model::{Filters, RepositoryStats};
use gitscope::output::json;
use gitscope::output::markdown::{self, Mode};
use gitscope::output::terminal::{self, TerminalOptions};
use gitscope::output::variables::{Context, VariableRegistry};
use gitscope::{analyze_path, is_remote};

const FILES: usize = 20;
const LINES: usize = 200;

fn from_env(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(default)
}

fn build_repository(commits: usize) -> TestRepo {
    let repo = TestRepo::named("bench-project");
    let authors = [
        ("Alice", "alice@example.com"),
        ("Bob", "bob@example.com"),
        ("Carol", "carol@example.com"),
        ("Dave", "dave@example.com"),
    ];
    let mut contents: Vec<Vec<String>> = (0..FILES)
        .map(|f| {
            (0..LINES)
                .map(|l| format!("line {l} of file {f}"))
                .collect()
        })
        .collect();

    for i in 0..commits {
        let (name, email) = authors[i % authors.len()];
        let file = i % FILES;
        let lines = &mut contents[file];
        for k in 0..3 {
            let at = (i * 7 + k * 13) % lines.len();
            lines[at] = format!("edited by commit {i} (change {k})");
        }
        lines.push(format!("added by commit {i}"));

        let month = 1 + (i / 90) % 12;
        let day = 1 + i % 28;
        let hour = i % 24;
        repo.by(name, email)
            .at(&format!("2026-{month:02}-{day:02} {hour:02}:30"))
            .message(&format!("commit {i}: change file{file:02}.txt"))
            .write(
                &format!("src/dir{}/file{file:02}.txt", file % 4),
                &lines.join("\n"),
            )
            .commit();
    }
    repo
}

fn measure(runs: usize, mut work: impl FnMut()) -> (Duration, Duration) {
    work();
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            work();
            start.elapsed()
        })
        .collect();
    times.sort();
    (times[0], times[times.len() / 2])
}

fn millis(duration: Duration) -> String {
    format!("{:>9.2} ms", duration.as_secs_f64() * 1000.0)
}

fn report(name: &str, runs: usize, work: impl FnMut()) {
    let (best, median) = measure(runs, work);
    println!("{name:<44}{}{}", millis(best), millis(median));
}

fn main() {
    let commits = from_env("GITSCOPE_BENCH_COMMITS", 1000);
    let runs = from_env("GITSCOPE_BENCH_RUNS", 7);

    println!("Building a repository with {commits} commits and {FILES} files...");
    let started = Instant::now();
    let repo = build_repository(commits);
    println!("done in {:.1?}\n", started.elapsed());
    let path = repo.path().to_path_buf();
    assert!(!is_remote(&path));

    println!(
        "gitscope {} · {commits} commits · {runs} runs each",
        env!("CARGO_PKG_VERSION")
    );
    println!("{:<44}{:>12}{:>12}", "scenario", "best", "median");
    println!("{}", "-".repeat(68));

    let nobody = Filters {
        author: Some("nobody@nowhere.invalid".into()),
        ..Filters::default()
    };
    report("walk only (no commit matches, no diffs)", runs, || {
        black_box(analyze_path(&path, nobody.clone(), Options::default()).unwrap());
    });

    report("full report (--json data, no details)", runs, || {
        black_box(analyze_path(&path, Filters::default(), Options::default()).unwrap());
    });

    report("full report with per-commit details (--all)", runs, || {
        let options = Options {
            commit_details: true,
        };
        black_box(analyze_path(&path, Filters::default(), options).unwrap());
    });

    let one_author = Filters {
        author: Some("alice".into()),
        ..Filters::default()
    };
    report("one author (diffs only their commits)", runs, || {
        black_box(analyze_path(&path, one_author.clone(), Options::default()).unwrap());
    });

    let stats: RepositoryStats = analyze_path(
        &path,
        Filters::default(),
        Options {
            commit_details: true,
        },
    )
    .unwrap();

    report("render: terminal report", runs, || {
        black_box(terminal::render(&stats, &TerminalOptions::default()));
    });
    report("render: JSON (with per-commit details)", runs, || {
        black_box(json::render(&stats).unwrap());
    });

    let template = "\
# Project

<!-- gitscope:start -->
{commits} commits by {contributors} people since {first_commit}.
{insertions} lines added, {deletions} removed. Top: {top_author}.
<!-- gitscope:end -->
";
    let registry = VariableRegistry::standard();
    let options = markdown::Options {
        mode: Mode::Update,
        ..markdown::Options::default()
    };
    report("render: Markdown template (update mode)", runs, || {
        let context = Context::new(&stats);
        black_box(markdown::render(template, &registry, &context, &options).unwrap());
    });
}
