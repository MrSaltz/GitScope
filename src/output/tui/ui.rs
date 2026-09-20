use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap};

use super::app::{App, SaveStatus, Tab};
use super::i18n::{Texts, fill};
use crate::config::Language;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::model::{ChangeKind, CommitInfo, FileChange, Filters, RepositoryStats};
use crate::output::format::{bar_with_count, day, short_hash, signed, sparkline, thousands};
use crate::output::terminal::hour_axis;

const WIDE: u16 = 100;
const PERCENT_BAR: usize = 20;
const LABEL: usize = 20;

fn accent() -> Style {
    Style::new().fg(Color::Cyan)
}

fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

fn bold() -> Style {
    Style::new().add_modifier(Modifier::BOLD)
}

fn selected() -> Style {
    Style::new().add_modifier(Modifier::REVERSED)
}

fn titled(title: impl AsRef<str>) -> Block<'static> {
    Block::bordered().title(Line::from(format!(" {} ", title.as_ref())).style(accent()))
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let t = app.texts();
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_tabs(frame, header, app, t);
    match app.tab() {
        Tab::Configurations => draw_configurations(frame, body, app, t),
        _ if app.stats.commits.total == 0 => draw_empty(frame, body, &app.stats.filters, t),
        Tab::Overview => draw_overview(frame, body, &app.stats, t),
        Tab::Contributors => draw_contributors(frame, body, app, t),
        Tab::Files => draw_files(frame, body, app, t),
        Tab::Activity => draw_activity(frame, body, &app.stats, t),
        Tab::Commits => draw_commits(frame, body, app, t),
    }
    draw_footer(frame, footer, app, t);
    if app.help_open() {
        draw_help(frame, t);
    }
}

fn describe_filters(filters: &Filters, t: &Texts) -> String {
    let mut parts = Vec::new();
    if let Some(author) = &filters.author {
        parts.push(format!("{} \"{author}\"", t.filter_author));
    }
    if let Some(branch) = &filters.branch {
        parts.push(format!("{} {branch}", t.filter_branch));
    }
    if let Some(since) = filters.since {
        parts.push(format!("{} {since}", t.filter_since));
    }
    if let Some(until) = filters.until {
        parts.push(format!("{} {until}", t.filter_until));
    }
    parts.join(" · ")
}

fn draw_tabs(frame: &mut Frame, area: Rect, app: &App, t: &Texts) {
    let stats = &app.stats;
    let filters = describe_filters(&stats.filters, t);
    let mut title = format!(
        "GITSCOPE v{} · {}",
        env!("CARGO_PKG_VERSION"),
        stats.repository.name
    );
    if !filters.is_empty() {
        title.push_str(&format!(" · {filters}"));
    }
    let tabs = Tabs::new(
        t.tabs
            .iter()
            .enumerate()
            .map(|(i, name)| format!("{} {name}", i + 1)),
    )
    .select(app.tab().index())
    .highlight_style(accent().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
    .divider("│")
    .block(titled(title));
    frame.render_widget(tabs, area);
}

fn draw_empty(frame: &mut Frame, area: Rect, filters: &Filters, t: &Texts) {
    let text = if *filters == Filters::default() {
        t.no_commits_yet
    } else {
        t.no_commits_match
    };
    frame.render_widget(
        Paragraph::new(text).centered().block(Block::bordered()),
        area,
    );
}

fn kv(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<LABEL$}"), dim()),
        Span::styled(value, bold()),
    ])
}

fn draw_overview(frame: &mut Frame, area: Rect, stats: &RepositoryStats, t: &Texts) {
    let [top, bottom] = Layout::vertical([Constraint::Length(12), Constraint::Min(0)]).areas(area);
    let [facts, languages] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);

    let commits = &stats.commits;
    let mut lines = vec![
        kv(t.commits, thousands(commits.total)),
        kv(t.contributors, thousands(stats.contributors.len())),
        kv(t.branches, thousands(stats.repository.branches)),
        kv(t.tags, thousands(stats.repository.tags)),
        kv(t.files, thousands(stats.files.total)),
        kv(t.insertions, signed('+', commits.insertions)),
        kv(t.deletions, signed('-', commits.deletions)),
    ];
    if let (Some(first), Some(last)) = (commits.first, commits.last) {
        lines.push(kv(t.period, format!("{} → {}", day(first), day(last))));
    }
    if let Some(hour) = commits.most_active_hour {
        lines.push(kv(t.most_active_hour, format!("{hour:02}:00 UTC")));
    }
    frame.render_widget(Paragraph::new(lines).block(titled(t.repository)), facts);

    let language_lines: Vec<Line> = if stats.languages.is_empty() {
        vec![Line::styled(t.no_language, dim())]
    } else {
        stats
            .languages
            .iter()
            .map(|l| {
                let length = ((l.percentage / 100.0 * PERCENT_BAR as f64).round() as usize).max(1);
                Line::from(vec![
                    Span::raw(format!("{:<14}{:>5.1}%  ", l.name, l.percentage)),
                    Span::styled("█".repeat(length), accent()),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(language_lines).block(titled(t.languages)),
        languages,
    );

    let rows = stats.contributors.iter().map(|c| {
        Row::new(vec![
            Cell::from(c.name.clone()),
            Cell::from(fill(t.n_commits, &[("n", &thousands(c.commits))])),
            Cell::from(format!("{:.1}%", c.percentage)),
            Cell::from(signed('+', c.insertions)),
            Cell::from(signed('-', c.deletions)),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Min(16),
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .block(titled(t.top_contributors));
    frame.render_widget(table, bottom);
}

fn draw_contributors(frame: &mut Frame, area: Rect, app: &mut App, t: &Texts) {
    let header = Row::new([
        t.col_name,
        t.col_email,
        t.col_commits,
        "%",
        t.col_files,
        t.col_plus_lines,
        t.col_minus_lines,
        t.col_first,
        t.col_last,
    ])
    .style(bold());
    let rows: Vec<Row> = app
        .stats
        .contributors
        .iter()
        .map(|c| {
            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(c.email.clone()),
                Cell::from(thousands(c.commits)),
                Cell::from(format!("{:.1}", c.percentage)),
                Cell::from(thousands(c.files_changed)),
                Cell::from(thousands(c.insertions)),
                Cell::from(thousands(c.deletions)),
                Cell::from(day(c.first_commit)),
                Cell::from(day(c.last_commit)),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Min(14),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .row_highlight_style(selected())
    .highlight_symbol("▶ ")
    .block(titled(fill(
        t.contributors_title,
        &[("n", &app.stats.contributors.len())],
    )));
    frame.render_stateful_widget(table, area, &mut app.contributors);
}

fn draw_files(frame: &mut Frame, area: Rect, app: &mut App, t: &Texts) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)]).areas(area);

    let rows: Vec<Row> = app
        .stats
        .files
        .most_modified
        .iter()
        .map(|f| {
            Row::new(vec![
                Cell::from(f.path.clone()),
                Cell::from(thousands(f.modifications)),
            ])
        })
        .collect();
    let table = Table::new(rows, [Constraint::Min(20), Constraint::Length(9)])
        .header(Row::new([t.col_path, t.col_changes]).style(bold()))
        .row_highlight_style(selected())
        .highlight_symbol("▶ ")
        .block(titled(t.most_modified));
    frame.render_stateful_widget(table, left, &mut app.files);

    let mut lines = vec![
        kv(t.total_files, thousands(app.stats.files.total)),
        Line::raw(""),
    ];
    lines.extend(app.stats.files.extensions.iter().map(|e| {
        let name = e
            .extension
            .as_ref()
            .map_or_else(|| t.no_extension.to_owned(), |ext| format!(".{ext}"));
        kv(&name, thousands(e.count))
    }));
    frame.render_widget(Paragraph::new(lines).block(titled(t.extensions)), right);
}

fn draw_activity(frame: &mut Frame, area: Rect, stats: &RepositoryStats, t: &Texts) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);

    let months = &stats.activity.by_month;
    let fit = (left.height as usize).saturating_sub(2).max(1);
    let shown = &months[months.len().saturating_sub(fit)..];
    let max = shown.iter().map(|m| m.commits).max().unwrap_or(0);
    let bar_width = (left.width as usize).saturating_sub(20).clamp(5, 60);
    let lines: Vec<Line> = shown
        .iter()
        .map(|m| {
            Line::from(format!(
                "{}  {}",
                m.month,
                bar_with_count(m.commits, max, bar_width)
            ))
        })
        .collect();
    let title = if shown.len() < months.len() {
        fill(
            t.per_month_last,
            &[("shown", &shown.len()), ("total", &months.len())],
        )
    } else {
        t.per_month.to_owned()
    };
    frame.render_widget(Paragraph::new(lines).block(titled(title)), left);

    let [weekdays, hours] =
        Layout::vertical([Constraint::Length(9), Constraint::Min(0)]).areas(right);
    let max = stats
        .commits
        .by_weekday
        .iter()
        .map(|d| d.commits)
        .max()
        .unwrap_or(0);
    let lines: Vec<Line> = stats
        .commits
        .by_weekday
        .iter()
        .enumerate()
        .map(|(i, d)| {
            Line::from(format!(
                "{}  {}",
                t.weekdays[i],
                bar_with_count(d.commits, max, 20)
            ))
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).block(titled(t.by_weekday)), weekdays);

    let counts: Vec<usize> = stats.commits.by_hour.iter().map(|h| h.commits).collect();
    let lines = vec![
        Line::styled(sparkline(&counts), accent()),
        Line::styled(hour_axis(), dim()),
    ];
    frame.render_widget(Paragraph::new(lines).block(titled(t.by_hour)), hours);
}

fn draw_commits(frame: &mut Frame, area: Rect, app: &mut App, t: &Texts) {
    let [list, detail] = if area.width >= WIDE {
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area)
    } else {
        Layout::vertical([Constraint::Min(6), Constraint::Length(12)]).areas(area)
    };

    let rows: Vec<Row> = app
        .visible_commits()
        .map(|c| {
            Row::new(vec![
                Cell::from(c.date.format("%Y-%m-%d %H:%M").to_string()),
                Cell::from(short_hash(&c.hash).to_owned()),
                Cell::from(c.author.clone()),
                Cell::from(c.message.clone()),
            ])
        })
        .collect();
    let title = if app.filter().is_empty() {
        fill(t.commits_title, &[("n", &app.visible_count())])
    } else {
        fill(
            t.commits_filtered,
            &[
                ("shown", &app.visible_count()),
                ("total", &app.total_commit_details()),
                ("filter", &app.filter()),
            ],
        )
    };
    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(7),
            Constraint::Length(14),
            Constraint::Min(10),
        ],
    )
    .header(Row::new([t.col_date, t.col_hash, t.col_author, t.col_message]).style(bold()))
    .row_highlight_style(selected())
    .highlight_symbol("▶ ")
    .block(titled(title));
    frame.render_stateful_widget(table, list, &mut app.commits);

    let lines = app
        .selected_commit()
        .map(|c| commit_lines(c, detail.width as usize, detail.height as usize, t))
        .unwrap_or_else(|| vec![Line::styled(t.no_commit_selected, dim())]);
    frame.render_widget(Paragraph::new(lines).block(titled(t.details)), detail);
}

pub(super) fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let mut word = word;
        while word.width() > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let (head, tail) = split_at_width(word, width);
            lines.push(head.to_owned());
            word = tail;
        }
        if word.is_empty() {
            continue;
        }
        let space = usize::from(!current.is_empty());
        if current.width() + space + word.width() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

fn split_at_width(text: &str, width: usize) -> (&str, &str) {
    let mut used = 0;
    for (index, ch) in text.char_indices() {
        let w = ch.width().unwrap_or(0);
        if used + w > width && index > 0 {
            return text.split_at(index);
        }
        used += w;
    }
    (text, "")
}

fn kv_wrapped(label: &str, value: &str, width: usize) -> Vec<Line<'static>> {
    let value_width = width.saturating_sub(LABEL).max(8);
    wrap(value, value_width)
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            let label = if index == 0 {
                format!("{label:<LABEL$}")
            } else {
                " ".repeat(LABEL)
            };
            Line::from(vec![
                Span::styled(label, dim()),
                Span::styled(chunk, bold()),
            ])
        })
        .collect()
}

fn change_lines(change: &FileChange, width: usize) -> Vec<Line<'static>> {
    let (symbol, color) = match change.kind {
        ChangeKind::Added => ('+', Color::Green),
        ChangeKind::Modified => ('M', Color::Yellow),
        ChangeKind::Deleted => ('-', Color::Red),
        ChangeKind::Renamed => ('R', Color::Cyan),
    };
    let path = match &change.old_path {
        Some(old) => format!("{old} -> {}", change.path),
        None => change.path.clone(),
    };
    wrap(&path, width.saturating_sub(2).max(8))
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            let lead = if index == 0 {
                Span::styled(format!("{symbol} "), Style::new().fg(color))
            } else {
                Span::raw("  ")
            };
            Line::from(vec![lead, Span::raw(chunk)])
        })
        .collect()
}

fn commit_lines(commit: &CommitInfo, width: usize, height: usize, t: &Texts) -> Vec<Line<'static>> {
    let inner = width.saturating_sub(2);
    let mut lines = Vec::new();
    lines.push(kv(t.label_hash, short_hash(&commit.hash).to_owned()));
    lines.extend(kv_wrapped(
        t.label_author,
        &format!("{} <{}>", commit.author, commit.email),
        inner,
    ));
    lines.push(kv(
        t.label_date,
        commit.date.format("%Y-%m-%d %H:%M UTC").to_string(),
    ));
    lines.extend(kv_wrapped(t.label_message, &commit.message, inner));
    lines.push(kv(t.label_files_changed, thousands(commit.files_changed)));
    lines.push(kv(t.insertions, signed('+', commit.insertions)));
    lines.push(kv(t.deletions, signed('-', commit.deletions)));
    lines.push(Line::raw(""));

    let budget = height.saturating_sub(2 + lines.len());
    let total = commit.changes.len();
    let mut used = 0;
    let mut shown = 0;
    for (index, change) in commit.changes.iter().enumerate() {
        let rows = change_lines(change, inner);
        let reserve = usize::from(index + 1 < total);
        if used + rows.len() + reserve > budget {
            break;
        }
        used += rows.len();
        shown += 1;
        lines.extend(rows);
    }
    if shown < total {
        lines.push(Line::styled(
            fill(t.and_more_files, &[("n", &(total - shown))]),
            dim(),
        ));
    }
    lines
}

fn draw_configurations(frame: &mut Frame, area: Rect, app: &mut App, t: &Texts) {
    let block = titled(t.config_title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [table_area, _, hint_area, status_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    let current = app.language();
    let options: Vec<Span> = Language::ALL
        .iter()
        .flat_map(|&language| {
            let (mark, style) = if language == current {
                ("●", accent().add_modifier(Modifier::BOLD))
            } else {
                ("○", dim())
            };
            [Span::styled(
                format!("{mark} {}   ", language.native_name()),
                style,
            )]
        })
        .collect();
    let table = Table::new(
        [Row::new(vec![
            Cell::from(t.setting_language),
            Cell::from(Line::from(options)),
        ])],
        [Constraint::Length(20), Constraint::Min(20)],
    )
    .header(Row::new([t.col_setting, t.col_value]).style(bold()))
    .row_highlight_style(selected())
    .highlight_symbol("▶ ");

    let status = match app.save_status() {
        SaveStatus::Idle => Line::raw(""),
        SaveStatus::Saved => {
            let path = app
                .config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            Line::styled(
                fill(t.saved_to, &[("path", &path)]),
                Style::new().fg(Color::Green),
            )
        }
        SaveStatus::NotStored => Line::styled(t.not_stored, Style::new().fg(Color::Yellow)),
        SaveStatus::Failed(error) => Line::styled(
            fill(t.save_failed, &[("error", error)]),
            Style::new().fg(Color::Red),
        ),
    };

    frame.render_stateful_widget(table, table_area, &mut app.settings);
    frame.render_widget(
        Paragraph::new(Line::styled(t.config_hint, dim())),
        hint_area,
    );
    frame.render_widget(
        Paragraph::new(status).wrap(Wrap { trim: true }),
        status_area,
    );
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, t: &Texts) {
    let line = if app.is_editing_filter() {
        Line::from(vec![
            Span::styled("/ ", accent()),
            Span::raw(app.filter().to_owned()),
            Span::styled("█", accent()),
            Span::styled(format!("   {}", t.editing_hint), dim()),
        ])
    } else {
        let base = if app.tab() == Tab::Configurations {
            t.hints_config
        } else {
            t.hints
        };
        let mut hints = format!(" {base}");
        if !app.filter().is_empty() {
            hints.push_str(&format!(" · {}", t.hint_clear_filter));
        }
        Line::styled(hints, dim())
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_help(frame: &mut Frame, t: &Texts) {
    let mut lines = vec![Line::raw("")];
    lines.extend(t.help_keys.iter().map(|(key, what)| {
        Line::from(vec![
            Span::styled(format!("  {key:<18}"), accent()),
            Span::raw((*what).to_owned()),
        ])
    }));
    lines.push(Line::raw(""));
    lines.push(Line::styled(format!("  {}", t.help_close), dim()));

    let area = centered(frame.area(), 72, lines.len() as u16 + 2);
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(titled(t.help_title)), area);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}
