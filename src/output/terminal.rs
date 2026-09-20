use std::fmt::{self, Write};

use super::format::{
    bar_with_count, day, fit_left, fit_right, short_hash, signed, sparkline, thousands,
};
use crate::model::{ChangeKind, CommitInfo, ContributorStats, Filters, RepositoryStats};

const WIDTH: usize = 60;
const LABEL_WIDTH: usize = 20;
const VALUE_WIDTH: usize = 24;
const BAR_WIDTH: usize = 30;
const PERCENT_BAR_WIDTH: usize = 20;
const MAX_ACTIVITY_MONTHS: usize = 24;

#[derive(Debug, Clone, Copy)]
pub struct TerminalOptions {
    pub top: usize,
    pub verbose: bool,
}

impl Default for TerminalOptions {
    fn default() -> Self {
        Self {
            top: 10,
            verbose: false,
        }
    }
}

pub fn render(stats: &RepositoryStats, options: &TerminalOptions) -> String {
    let mut out = String::new();
    write_report(&mut out, stats, options).expect("writing to a String cannot fail");
    out
}

fn write_report(
    out: &mut String,
    stats: &RepositoryStats,
    options: &TerminalOptions,
) -> fmt::Result {
    write_header(out, stats)?;

    if stats.commits.total == 0 {
        return if stats.filters == Filters::default() {
            writeln!(out, "This repository has no commits yet.")
        } else {
            writeln!(out, "No commits match the given filters.")
        };
    }

    if stats.filters.author.is_some() {
        write_author_view(out, stats, options)?;
    } else {
        write_overview(out, stats, options)?;
    }
    if let Some(details) = &stats.commit_details {
        write_commit_list(out, stats, details, options)?;
    }
    Ok(())
}

fn write_header(out: &mut String, stats: &RepositoryStats) -> fmt::Result {
    let inner = WIDTH - 2;
    writeln!(out, "╭{}╮", "─".repeat(inner))?;
    writeln!(
        out,
        "│{}│",
        center(&format!("GITSCOPE v{}", env!("CARGO_PKG_VERSION")), inner)
    )?;
    writeln!(
        out,
        "│{}│",
        center(&fit_right(&stats.repository.name, inner - 2), inner)
    )?;
    writeln!(out, "╰{}╯", "─".repeat(inner))?;
    writeln!(out)?;

    let filters = describe_filters(&stats.filters);
    if !filters.is_empty() {
        writeln!(out, "Filters: {filters}")?;
        writeln!(out)?;
    }
    Ok(())
}

pub(super) fn describe_filters(filters: &Filters) -> String {
    let mut parts = Vec::new();
    if let Some(author) = &filters.author {
        parts.push(format!("author \"{author}\""));
    }
    if let Some(branch) = &filters.branch {
        parts.push(format!("branch {branch}"));
    }
    if let Some(since) = filters.since {
        parts.push(format!("since {since}"));
    }
    if let Some(until) = filters.until {
        parts.push(format!("until {until}"));
    }
    parts.join(" · ")
}

fn write_overview(
    out: &mut String,
    stats: &RepositoryStats,
    options: &TerminalOptions,
) -> fmt::Result {
    write_repository(out, stats)?;
    write_commits(out, stats)?;
    write_top_contributors(out, stats, options)?;
    write_files(out, stats, options)?;
    write_languages(out, stats)?;
    write_activity(out, stats)
}

fn write_repository(out: &mut String, stats: &RepositoryStats) -> fmt::Result {
    section(out, "Repository")?;
    row(out, "Commits", &thousands(stats.commits.total))?;
    row(out, "Contributors", &thousands(stats.contributors.len()))?;
    row(out, "Branches", &thousands(stats.repository.branches))?;
    row(out, "Tags", &thousands(stats.repository.tags))?;
    if let (Some(first), Some(last)) = (stats.commits.first, stats.commits.last) {
        row(out, "Period", &format!("{} → {}", day(first), day(last)))?;
    }
    writeln!(out)
}

fn write_commits(out: &mut String, stats: &RepositoryStats) -> fmt::Result {
    let commits = &stats.commits;
    section(out, "Commits")?;
    row(out, "Total", &thousands(commits.total))?;
    if let Some(first) = commits.first {
        row(out, "First commit", &day(first))?;
    }
    if let Some(last) = commits.last {
        row(out, "Last commit", &day(last))?;
    }
    if let Some(hour) = commits.most_active_hour {
        row(out, "Most active hour", &format!("{hour:02}:00"))?;
    }

    writeln!(out)?;
    writeln!(out, "  By weekday")?;
    let max = commits
        .by_weekday
        .iter()
        .map(|d| d.commits)
        .max()
        .unwrap_or(0);
    for d in &commits.by_weekday {
        writeln!(
            out,
            "  {}  {}",
            d.weekday,
            bar_with_count(d.commits, max, BAR_WIDTH)
        )?;
    }

    writeln!(out)?;
    let hours: Vec<usize> = commits.by_hour.iter().map(|h| h.commits).collect();
    writeln!(out, "  By hour (UTC)")?;
    writeln!(out, "  {}", sparkline(&hours))?;
    writeln!(out, "  {}", hour_axis())?;
    writeln!(out)
}

fn write_top_contributors(
    out: &mut String,
    stats: &RepositoryStats,
    options: &TerminalOptions,
) -> fmt::Result {
    section(out, "Top contributors")?;
    for c in stats.contributors.iter().take(options.top) {
        let unit = if c.commits == 1 { "commit " } else { "commits" };
        writeln!(
            out,
            "  {:<16} {:>6} {unit} {:>5.1}% {:>9} {:>8}",
            fit_right(&c.name, 16),
            thousands(c.commits),
            c.percentage,
            signed('+', c.insertions),
            signed('-', c.deletions),
        )?;
        if options.verbose {
            writeln!(
                out,
                "    {} · {} files changed",
                c.email,
                thousands(c.files_changed)
            )?;
        }
    }
    more_note(out, stats.contributors.len(), options.top)?;
    writeln!(out)
}

fn write_files(
    out: &mut String,
    stats: &RepositoryStats,
    options: &TerminalOptions,
) -> fmt::Result {
    let files = &stats.files;
    section(out, "Files")?;
    row(out, "Total files", &thousands(files.total))?;

    if !files.extensions.is_empty() {
        writeln!(out)?;
        writeln!(out, "  Extensions")?;
        for e in files.extensions.iter().take(options.top) {
            let name = e
                .extension
                .as_ref()
                .map_or_else(|| "(none)".to_owned(), |ext| format!(".{ext}"));
            writeln!(
                out,
                "  {:<12} {:>6}",
                fit_right(&name, 12),
                thousands(e.count)
            )?;
        }
        more_note(out, files.extensions.len(), options.top)?;
    }

    write_most_modified(out, stats, options)?;
    writeln!(out)
}

fn write_most_modified(
    out: &mut String,
    stats: &RepositoryStats,
    options: &TerminalOptions,
) -> fmt::Result {
    let files = &stats.files;
    if files.most_modified.is_empty() {
        return Ok(());
    }
    writeln!(out)?;
    writeln!(out, "  Most modified files")?;
    for f in files.most_modified.iter().take(options.top) {
        writeln!(
            out,
            "  {:<46} {:>8}",
            fit_left(&f.path, 46),
            thousands(f.modifications)
        )?;
    }
    more_note(out, files.most_modified.len(), options.top)
}

fn write_languages(out: &mut String, stats: &RepositoryStats) -> fmt::Result {
    if stats.languages.is_empty() {
        return Ok(());
    }
    section(out, "Languages")?;
    for l in &stats.languages {
        let length = ((l.percentage / 100.0 * PERCENT_BAR_WIDTH as f64).round() as usize).max(1);
        writeln!(
            out,
            "  {:<14} {:>5.1}%  {}",
            fit_right(&l.name, 14),
            l.percentage,
            "█".repeat(length)
        )?;
    }
    writeln!(out)
}

fn write_activity(out: &mut String, stats: &RepositoryStats) -> fmt::Result {
    let months = &stats.activity.by_month;
    if months.is_empty() {
        return Ok(());
    }
    section(out, "Activity")?;
    let hidden = months.len().saturating_sub(MAX_ACTIVITY_MONTHS);
    if hidden > 0 {
        writeln!(
            out,
            "  (showing the last {MAX_ACTIVITY_MONTHS} of {} months)",
            months.len()
        )?;
    }
    let shown = &months[hidden..];
    let max = shown.iter().map(|m| m.commits).max().unwrap_or(0);
    for m in shown {
        writeln!(
            out,
            "  {}  {}",
            m.month,
            bar_with_count(m.commits, max, BAR_WIDTH)
        )?;
    }
    writeln!(out)
}

fn write_author_view(
    out: &mut String,
    stats: &RepositoryStats,
    options: &TerminalOptions,
) -> fmt::Result {
    for contributor in &stats.contributors {
        write_contributor(out, contributor)?;
    }
    write_activity(out, stats)?;

    if !stats.files.most_modified.is_empty() {
        section(out, "Most modified files")?;
        for f in stats.files.most_modified.iter().take(options.top) {
            writeln!(
                out,
                "  {:<46} {:>8}",
                fit_left(&f.path, 46),
                thousands(f.modifications)
            )?;
        }
        more_note(out, stats.files.most_modified.len(), options.top)?;
        writeln!(out)?;
    }
    Ok(())
}

fn write_contributor(out: &mut String, c: &ContributorStats) -> fmt::Result {
    section(out, "Contributor")?;
    row(out, "Name", &c.name)?;
    row(out, "Email", &c.email)?;
    row(out, "Commits", &thousands(c.commits))?;
    row(out, "Files changed", &thousands(c.files_changed))?;
    row(out, "Insertions", &signed('+', c.insertions))?;
    row(out, "Deletions", &signed('-', c.deletions))?;
    row(out, "First commit", &day(c.first_commit))?;
    row(out, "Last commit", &day(c.last_commit))?;
    writeln!(out)
}

fn write_commit_list(
    out: &mut String,
    stats: &RepositoryStats,
    details: &[CommitInfo],
    options: &TerminalOptions,
) -> fmt::Result {
    let title = match &stats.filters.author {
        Some(author) => format!("Commits by {author}"),
        None => "Commit history".to_owned(),
    };
    section(out, &title)?;
    writeln!(out)?;

    for commit in details {
        if options.verbose {
            write_commit_detail(out, commit)?;
        } else {
            writeln!(
                out,
                "{}  {}  {}",
                commit.date.format("%Y-%m-%d"),
                commit.date.format("%H:%M"),
                short_hash(&commit.hash)
            )?;
            writeln!(out, "{}", commit.message)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn write_commit_detail(out: &mut String, commit: &CommitInfo) -> fmt::Result {
    let rule = "─".repeat(40);
    writeln!(out, "{}", short_hash(&commit.hash))?;
    writeln!(out, "{rule}")?;
    writeln!(out, "{:<12} {} <{}>", "Author", commit.author, commit.email)?;
    writeln!(
        out,
        "{:<12} {}",
        "Date",
        commit.date.format("%Y-%m-%d %H:%M")
    )?;
    writeln!(out, "{:<12} {}", "Message", commit.message)?;

    if !commit.changes.is_empty() {
        writeln!(out)?;
        writeln!(out, "Changes")?;
        for change in &commit.changes {
            let symbol = match change.kind {
                ChangeKind::Added => '+',
                ChangeKind::Modified => 'M',
                ChangeKind::Deleted => '-',
                ChangeKind::Renamed => 'R',
            };
            match &change.old_path {
                Some(old) => writeln!(out, "  {symbol} {old} -> {}", change.path)?,
                None => writeln!(out, "  {symbol} {}", change.path)?,
            }
        }
    }

    writeln!(out)?;
    writeln!(
        out,
        "{:<14}{:>6}",
        "Files changed",
        thousands(commit.files_changed)
    )?;
    writeln!(
        out,
        "{:<14}{:>6}",
        "Insertions",
        signed('+', commit.insertions)
    )?;
    writeln!(
        out,
        "{:<14}{:>6}",
        "Deletions",
        signed('-', commit.deletions)
    )
}

fn section(out: &mut String, title: &str) -> fmt::Result {
    writeln!(out, "{title}")?;
    writeln!(out, "{}", "─".repeat(WIDTH))
}

fn row(out: &mut String, label: &str, value: &str) -> fmt::Result {
    writeln!(out, "  {label:<LABEL_WIDTH$}{value:>VALUE_WIDTH$}")
}

fn more_note(out: &mut String, total: usize, shown: usize) -> fmt::Result {
    if total > shown {
        writeln!(
            out,
            "  … and {} more (raise --top to see them)",
            total - shown
        )?;
    }
    Ok(())
}

pub(super) fn hour_axis() -> String {
    let mut axis = [' '; 24];
    for (hour, label) in [(0, "0"), (6, "6"), (12, "12"), (18, "18")] {
        for (offset, ch) in label.chars().enumerate() {
            axis[hour + offset] = ch;
        }
    }
    axis.iter().collect()
}

fn center(text: &str, width: usize) -> String {
    let len = text.chars().count();
    let left = width.saturating_sub(len) / 2;
    let right = width.saturating_sub(len + left);
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hour_axis_is_24_characters() {
        let axis = hour_axis();
        assert_eq!(axis.chars().count(), 24);
        assert!(axis.starts_with("0     6     12    18"));
    }

    #[test]
    fn center_pads_both_sides() {
        assert_eq!(center("ab", 6), "  ab  ");
        assert_eq!(center("abc", 6), " abc  ");
        assert_eq!(center("toolong", 3), "toolong");
    }
}
