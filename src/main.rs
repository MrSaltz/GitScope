use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use anyhow::{Context as _, bail};
use clap::Parser;

use gitscope::analysis;
use gitscope::cli::{Cli, Command, MarkdownArgs, TemplateSourceArgs, TuiArgs, UpdateArgs};
use gitscope::config::{self, Config};
use gitscope::error::VariableProblem;
use gitscope::model::{Filters, RepositoryStats};
use gitscope::output::markdown::{self, Mode, UnknownPolicy};
use gitscope::output::variables::{Context, VariableRegistry};
use gitscope::output::{json, terminal, tui};
use gitscope::remote::{UrlKind, classify, redact};
use gitscope::repository::{CloneProgress, Repository};
use gitscope::{analyze_repository, is_remote, open_source};

fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        // O destino fechou o pipe (`gitscope ... | head`): não é erro.
        Err(err) if is_broken_pipe(&err) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        None => report(cli),
        Some(Command::Markdown(args)) => render_markdown(args),
        Some(Command::Update(args)) => update_markdown(args),
        Some(Command::Tui(args)) => run_tui(args),
        Some(Command::Variables) => {
            print_stdout(&VariableRegistry::standard().describe())?;
            Ok(())
        }
    }
}

fn report(cli: &Cli) -> anyhow::Result<()> {
    let filters = cli.filters();
    // Recusa argumentos ruins antes de gastar tempo (e banda) com um clone.
    filters.validate()?;

    let started = Instant::now();
    let repo = open_repository(&cli.path, cli.verbose)?;
    let stats = analyze_and_close(repo, filters, cli.analysis_options(), cli.verbose)?;

    if cli.verbose {
        eprintln!(
            "gitscope: commits matched: {}, contributors: {}, files in snapshot: {} ({:.0?})",
            stats.commits.total,
            stats.contributors.len(),
            stats.files.total,
            started.elapsed()
        );
        if stats.commits.total == 0 && cli.filter.author.is_some() {
            eprintln!("gitscope: warning: no commits match the given filters");
        }
    }

    let text = if cli.json {
        json::render(&stats).context("failed to serialise the report as JSON")?
    } else {
        let options = terminal::TerminalOptions {
            top: cli.top.get(),
            verbose: cli.verbose,
        };
        terminal::render(&stats, &options)
    };
    print_stdout(&text)?;
    Ok(())
}

fn run_tui(args: &TuiArgs) -> anyhow::Result<()> {
    if !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
        bail!(
            "the interactive interface needs a terminal (stdin and stdout must both be one); for pipes and files use the plain report or --json"
        );
    }
    let filters = args.filter.filters();
    filters.validate()?;

    let config_path = config::default_path();
    let config = match &config_path {
        Some(path) => Config::load(path).unwrap_or_else(|warning| {
            eprintln!("gitscope: warning: {warning}; using the defaults");
            Config::default()
        }),
        None => Config::default(),
    };

    eprintln!(
        "gitscope: analysing {}...",
        redact(&args.path.to_string_lossy())
    );
    let repo = open_repository(&args.path, false)?;
    let options = analysis::Options {
        commit_details: true,
    };
    let stats = analyze_and_close(repo, filters, options, false)?;
    tui::run(stats, config, config_path).context("the interactive interface failed")?;
    Ok(())
}

fn unknown_policy(args: &TemplateSourceArgs) -> UnknownPolicy {
    if args.keep_unknown {
        UnknownPolicy::Keep
    } else {
        UnknownPolicy::Error
    }
}

fn print_warnings(warnings: &[VariableProblem]) {
    for warning in warnings {
        eprintln!(
            "gitscope: warning: left {} untouched (line {})",
            warning.token, warning.line
        );
    }
}

fn render_markdown(args: &MarkdownArgs) -> anyhow::Result<()> {
    let filters = args.source.filter.filters();
    filters.validate()?;

    let template = markdown::read_document(&args.file)?;
    markdown::check_structure(&template, Mode::Render)?;
    let output = args.output.as_deref().filter(|p| p.as_os_str() != "-");
    if let Some(output) = output {
        if is_same_file(&args.file, output) {
            bail!(
                "refusing to overwrite the template {}: it is the output file. \
                 Use `gitscope update` to update a file in place, or choose another output",
                args.file.display()
            );
        }
    }

    let repo = open_repository(&args.source.repo, args.source.verbose)?;
    let stats = analyze_and_close(
        repo,
        filters,
        analysis::Options::default(),
        args.source.verbose,
    )?;

    let options = markdown::Options {
        mode: Mode::Render,
        unknown: unknown_policy(&args.source),
    };
    let rendered = markdown::render(
        &template,
        &VariableRegistry::standard(),
        &Context::new(&stats),
        &options,
    )?;
    print_warnings(&rendered.warnings);

    match output {
        Some(path) => {
            markdown::write_atomic(path, &rendered.text)?;
            eprintln!(
                "gitscope: wrote {} ({})",
                path.display(),
                counts(rendered.blocks, rendered.replacements)
            );
        }
        None => print_stdout(&rendered.text)?,
    }
    Ok(())
}

fn update_markdown(args: &UpdateArgs) -> anyhow::Result<()> {
    let filters = args.source.filter.filters();
    filters.validate()?;

    let check = |file: &Path| -> anyhow::Result<()> {
        let text = markdown::read_document(file)?;
        markdown::check_structure(&text, Mode::Update)?;
        Ok(())
    };
    if let Some(file) = &args.file {
        check(file)?;
    }

    let repo = open_repository(&args.source.repo, args.source.verbose)?;
    let file = match &args.file {
        Some(file) => file.clone(),
        None => {
            let root = repo.workdir().context(
                "this repository has no working directory (it is bare or remote), \
                 so there is no README.md to find: pass the file, as in `gitscope update FILE`",
            )?;
            let file = root.join("README.md");
            check(&file)?;
            file
        }
    };

    let stats = analyze_and_close(
        repo,
        filters,
        analysis::Options::default(),
        args.source.verbose,
    )?;
    let outcome = markdown::update_file(
        &file,
        &VariableRegistry::standard(),
        &Context::new(&stats),
        unknown_policy(&args.source),
    )?;
    print_warnings(&outcome.warnings);

    let what = counts(outcome.blocks, outcome.replacements);
    if outcome.changed {
        println!("Updated {} ({what})", file.display());
    } else {
        println!("{} is already up to date ({what})", file.display());
    }
    Ok(())
}

fn counts(blocks: usize, replacements: usize) -> String {
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    match blocks {
        0 => plural(replacements, "variable"),
        _ => format!(
            "{}, {}",
            plural(blocks, "block"),
            plural(replacements, "variable")
        ),
    }
}

fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn open_repository(source: &Path, verbose: bool) -> anyhow::Result<Repository> {
    let shown = redact(&source.to_string_lossy());
    if verbose {
        eprintln!("gitscope: analysing {shown}");
    }

    let will_clone = is_remote(source)
        && source
            .to_str()
            .is_some_and(|url| classify(url) == UrlKind::Supported);
    let repo = if will_clone {
        eprintln!("gitscope: cloning {shown} into a temporary directory...");
        let mut progress = ProgressLine::new();
        let opened = open_source(source, |update| progress.update(update));
        progress.finish();
        opened?
    } else {
        open_source(source, |_| {})?
    };
    if verbose && repo.is_temporary() {
        eprintln!("gitscope: temporary clone at {}", repo.location().display());
    }
    Ok(repo)
}

fn analyze_and_close(
    repo: Repository,
    filters: Filters,
    options: analysis::Options,
    verbose: bool,
) -> anyhow::Result<RepositoryStats> {
    let stats = analyze_repository(&repo, filters, options);

    let location = repo.location().to_path_buf();
    let temporary = repo.is_temporary();
    if let Err(err) = repo.close() {
        eprintln!(
            "gitscope: warning: could not remove the temporary clone {}: {err}",
            location.display()
        );
    } else if verbose && temporary {
        eprintln!("gitscope: temporary clone removed");
    }
    Ok(stats?)
}

fn print_stdout(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(text.as_bytes())?;
    stdout.flush()
}

fn is_broken_pipe(err: &anyhow::Error) -> bool {
    err.downcast_ref::<io::Error>()
        .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
}

struct ProgressLine {
    enabled: bool,
    last_percent: Option<usize>,
    drawn: bool,
}

impl ProgressLine {
    fn new() -> Self {
        Self {
            enabled: io::stderr().is_terminal(),
            last_percent: None,
            drawn: false,
        }
    }

    fn update(&mut self, progress: CloneProgress) {
        if !self.enabled || progress.total_objects == 0 {
            return;
        }
        let percent = progress.received_objects * 100 / progress.total_objects;
        if self.last_percent == Some(percent) {
            return;
        }
        self.last_percent = Some(percent);
        self.drawn = true;
        eprint!(
            "\r  Receiving objects: {percent:>3}% ({}/{}), {:.1} MiB",
            progress.received_objects,
            progress.total_objects,
            progress.received_bytes as f64 / (1024.0 * 1024.0)
        );
    }

    fn finish(&self) {
        if self.drawn {
            eprintln!();
        }
    }
}
