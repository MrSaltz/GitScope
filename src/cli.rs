use std::num::NonZeroUsize;
use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

use crate::analysis;
use crate::model::Filters;

/// Statistics for Git repositories, as a terminal report, JSON, Markdown or an interactive interface.
#[derive(Debug, Parser)]
#[command(
    name = "gitscope",
    version,
    about = "A Rust CLI for analyzing Git repository statistics.",
    args_conflicts_with_subcommands = true,
    after_help = "\
Examples:
  gitscope .
  gitscope ./repo --since 2026-01-01 --until 2026-06-30
  gitscope ./repo --branch main
  gitscope ./repo --author \"Alice\" --all
  gitscope ./repo --json > stats.json
  gitscope https://github.com/owner/repo.git --since 2026-01-01

Interactive interface:
  gitscope tui ./repo              browse the statistics with the keyboard

Markdown templates (put {commits}, {contributors}... in your README):
  gitscope update                 refresh the <!-- gitscope:start --> blocks of README.md
  gitscope markdown README.md     print README.md with the variables resolved
  gitscope variables              list the available variables

Dates are calendar days in UTC and both bounds are inclusive."
)]
pub struct Cli {
    /// What to do instead of printing the report
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Repository to analyse: a local path (working directory, `.git` directory or
    /// bare repository) or an https://, http://, git:// or file:// URL, which is
    /// cloned into a temporary directory that is deleted afterwards
    #[arg(value_name = "PATH_OR_URL", default_value = ".")]
    pub path: PathBuf,

    #[command(flatten)]
    pub filter: FilterArgs,

    /// Print the report as JSON instead of formatted text
    #[arg(long)]
    pub json: bool,

    /// List every commit matching the filters, with its details
    #[arg(long)]
    pub all: bool,

    /// Print diagnostics to stderr; with --all, show full details of each commit
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Maximum rows in ranked lists of the formatted output (JSON is never truncated)
    #[arg(long, value_name = "N", default_value = "10")]
    pub top: NonZeroUsize,
}

/// The commands that work on Markdown files.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Resolve the {variables} of a Markdown template and print the result
    ///
    /// Inside <!-- gitscope:start --> / <!-- gitscope:end --> blocks only the
    /// blocks are resolved; a file without blocks is resolved as a whole. The
    /// template is never modified.
    Markdown(MarkdownArgs),

    /// Update the <!-- gitscope:start --> blocks of a Markdown file in place
    ///
    /// Only the inside of the blocks changes, atomically, and only if the file
    /// is valid. Each block keeps its template in an HTML comment so that it can
    /// be updated again later.
    Update(UpdateArgs),

    /// List the variables available in Markdown templates
    Variables,

    /// Browse the statistics in an interactive terminal interface
    ///
    /// Tabs for the overview, contributors, files, activity and commits, with a
    /// filterable commit list. Press ? inside for the keys.
    Tui(TuiArgs),
}

/// `gitscope tui`
#[derive(Debug, Args)]
pub struct TuiArgs {
    /// Repository to analyse: a local path or an https://, http://, git:// or
    /// file:// URL
    #[arg(value_name = "PATH_OR_URL", default_value = ".")]
    pub path: PathBuf,

    #[command(flatten)]
    pub filter: FilterArgs,
}

/// `gitscope markdown`
#[derive(Debug, Args)]
pub struct MarkdownArgs {
    /// Markdown file used as the template (it is never modified)
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Write the result to FILE instead of stdout (use - for stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    #[command(flatten)]
    pub source: TemplateSourceArgs,
}

/// `gitscope update`
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Markdown file to update (default: README.md at the root of the repository)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub source: TemplateSourceArgs,
}

/// Where the data of a template comes from, shared by `markdown` and `update`.
#[derive(Debug, Args)]
pub struct TemplateSourceArgs {
    /// Repository to analyse: a local path or an https://, http://, git:// or
    /// file:// URL
    #[arg(long, value_name = "PATH_OR_URL", default_value = ".")]
    pub repo: PathBuf,

    /// Leave unknown variables untouched and warn, instead of failing
    #[arg(long)]
    pub keep_unknown: bool,

    #[command(flatten)]
    pub filter: FilterArgs,

    /// Print diagnostics to stderr
    #[arg(long, short = 'v')]
    pub verbose: bool,
}

/// Restrictions on the analysed history, shared by every command.
#[derive(Debug, Clone, Default, Args)]
pub struct FilterArgs {
    /// Only commits authored on or after this day
    #[arg(long, value_name = "DATE", value_parser = parse_date)]
    pub since: Option<NaiveDate>,

    /// Only commits authored on or before this day
    #[arg(long, value_name = "DATE", value_parser = parse_date)]
    pub until: Option<NaiveDate>,

    /// Only commits whose author name or email contains this text (case-insensitive)
    #[arg(long, value_name = "AUTHOR")]
    pub author: Option<String>,

    /// Analyse this branch instead of HEAD
    #[arg(long, value_name = "BRANCH")]
    pub branch: Option<String>,
}

impl FilterArgs {
    pub fn filters(&self) -> Filters {
        Filters {
            since: self.since,
            until: self.until,
            author: self.author.clone(),
            branch: self.branch.clone(),
        }
    }
}

impl Cli {
    pub fn filters(&self) -> Filters {
        self.filter.filters()
    }

    pub fn analysis_options(&self) -> analysis::Options {
        analysis::Options {
            commit_details: self.all,
        }
    }
}

fn parse_date(input: &str) -> Result<NaiveDate, String> {
    Filters::parse_date(input).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("gitscope").chain(args.iter().copied()))
    }

    fn parse(args: &[&str]) -> Cli {
        try_parse(args).unwrap()
    }

    #[test]
    fn defaults() {
        let cli = parse(&[]);
        assert!(cli.command.is_none());
        assert_eq!(cli.path, PathBuf::from("."));
        assert_eq!(cli.top.get(), 10);
        assert_eq!(cli.filters(), Filters::default());
        assert!(!cli.json && !cli.all && !cli.verbose);
    }

    #[test]
    fn all_filters_combine() {
        let cli = parse(&[
            "./repo",
            "--author",
            "Widison",
            "--branch",
            "main",
            "--since",
            "2026-01-01",
            "--until",
            "2026-09-01",
            "--all",
            "--json",
            "-v",
            "--top",
            "3",
        ]);
        let filters = cli.filters();
        assert_eq!(filters.author.as_deref(), Some("Widison"));
        assert_eq!(filters.branch.as_deref(), Some("main"));
        assert_eq!(filters.since, NaiveDate::from_ymd_opt(2026, 1, 1));
        assert_eq!(filters.until, NaiveDate::from_ymd_opt(2026, 9, 1));
        assert!(cli.analysis_options().commit_details);
        assert!(cli.json && cli.verbose);
        assert_eq!(cli.top.get(), 3);
    }

    #[test]
    fn rejects_malformed_dates_and_zero_top() {
        assert!(try_parse(&["--since", "01/02/2026"]).is_err());
        assert!(try_parse(&["--until", "2026-13-40"]).is_err());
        assert!(try_parse(&["--top", "0"]).is_err());
    }

    #[test]
    fn markdown_subcommand() {
        let cli = parse(&[
            "markdown",
            "README.md",
            "-o",
            "out.md",
            "--repo",
            "../r",
            "--author",
            "alice",
            "--keep-unknown",
        ]);
        let Some(Command::Markdown(args)) = cli.command else {
            panic!("expected the markdown subcommand");
        };
        assert_eq!(args.file, PathBuf::from("README.md"));
        assert_eq!(args.output, Some(PathBuf::from("out.md")));
        assert_eq!(args.source.repo, PathBuf::from("../r"));
        assert!(args.source.keep_unknown);
        assert_eq!(
            args.source.filter.filters().author.as_deref(),
            Some("alice")
        );
    }

    #[test]
    fn update_subcommand_defaults_to_the_repository_readme() {
        let cli = parse(&["update"]);
        let Some(Command::Update(args)) = cli.command else {
            panic!("expected the update subcommand");
        };
        assert_eq!(args.file, None);
        assert_eq!(args.source.repo, PathBuf::from("."));
        assert!(!args.source.keep_unknown);

        let cli = parse(&["update", "docs/STATS.md", "--since", "2026-01-01"]);
        let Some(Command::Update(args)) = cli.command else {
            panic!("expected the update subcommand");
        };
        assert_eq!(args.file, Some(PathBuf::from("docs/STATS.md")));
        assert!(args.source.filter.since.is_some());
    }

    #[test]
    fn tui_subcommand() {
        let cli = parse(&[
            "tui",
            "./repo",
            "--author",
            "alice",
            "--since",
            "2026-01-01",
        ]);
        let Some(Command::Tui(args)) = cli.command else {
            panic!("expected the tui subcommand");
        };
        assert_eq!(args.path, PathBuf::from("./repo"));
        assert_eq!(args.filter.filters().author.as_deref(), Some("alice"));
        assert!(args.filter.since.is_some());

        let Some(Command::Tui(args)) = parse(&["tui"]).command else {
            panic!("expected the tui subcommand");
        };
        assert_eq!(args.path, PathBuf::from("."));
        // Report-only options do not apply to the interface.
        assert!(try_parse(&["tui", "--json"]).is_err());
    }

    #[test]
    fn variables_subcommand() {
        assert!(matches!(
            parse(&["variables"]).command,
            Some(Command::Variables)
        ));
    }

    #[test]
    fn report_options_cannot_be_mixed_with_subcommands() {
        assert!(try_parse(&["update", "--json"]).is_err());
        // Um nome de subcomando só é especial como primeiro argumento; depois de uma
        // opção é um caminho comum (e `./update` sempre é).
        let cli = parse(&["--json", "update"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.path, PathBuf::from("update"));
        assert!(parse(&["./update"]).command.is_none());
        assert!(try_parse(&["markdown"]).is_err());
    }
}
