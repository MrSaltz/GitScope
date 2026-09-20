use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::ui;
use super::{App, SaveStatus, Tab};
use crate::analysis::{self, testutil::record_with};
use crate::config::{Config, Language};
use crate::model::{Filters, RepositoryInfo, RepositoryStats};

fn info(name: &str) -> RepositoryInfo {
    RepositoryInfo {
        name: name.into(),
        branches: 2,
        tags: 1,
    }
}

fn stats() -> RepositoryStats {
    let mut first = record_with(
        "Alice",
        "alice@x.io",
        "2026-01-05 14:10",
        &["a.rs", "b.ts"],
        10,
        1,
    );
    first.message = "feat: first".into();
    let mut second = record_with("Bob", "bob@x.io", "2026-02-10 09:00", &["a.rs"], 2, 0);
    let mut third = record_with(
        "Alice",
        "alice@x.io",
        "2026-03-07 14:50",
        &["a.rs"],
        1_000,
        5,
    );
    third.message = "fix: last".into();
    let many: Vec<String> = (0..30).map(|i| format!("dir/file{i:02}.rs")).collect();
    let many: Vec<&str> = many.iter().map(String::as_str).collect();
    let mut fourth = record_with("Bob", "bob@x.io", "2026-03-08 10:00", &many, 30, 0);
    fourth.message = "chore: many files".into();
    for (record, id) in [&mut first, &mut second, &mut third, &mut fourth]
        .into_iter()
        .zip(["aa11", "bb22", "cc33", "dd44"])
    {
        record.id = id.repeat(10);
    }

    analysis::analyze(
        info("demo"),
        Filters::default(),
        [first, second, third, fourth].into_iter().map(Ok),
        ["a.rs", "b.ts", "README.md"].map(String::from),
        analysis::Options {
            commit_details: true,
        },
    )
    .unwrap()
}

fn app() -> App {
    App::new(stats())
}

fn empty_app() -> App {
    let stats = analysis::analyze(
        info("empty"),
        Filters::default(),
        std::iter::empty(),
        std::iter::empty(),
        analysis::Options {
            commit_details: true,
        },
    )
    .unwrap();
    App::new(stats)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn press(app: &mut App, keys: &[KeyCode]) {
    for &code in keys {
        app.handle_key(key(code));
    }
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        app.handle_key(key(KeyCode::Char(c)));
    }
}

fn screen(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| ui::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).map_or(" ", |cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn messages(app: &App) -> Vec<&str> {
    app.visible_commits().map(|c| c.message.as_str()).collect()
}

#[test]
fn starts_on_the_overview() {
    let app = app();
    assert_eq!(app.tab(), Tab::Overview);
    assert!(!app.should_quit());
    assert!(!app.help_open());
}

#[test]
fn tab_keys_cycle_through_the_six_screens_and_wrap() {
    let mut app = app();
    let mut seen = vec![app.tab()];
    for _ in 0..6 {
        press(&mut app, &[KeyCode::Tab]);
        seen.push(app.tab());
    }
    assert_eq!(
        seen,
        vec![
            Tab::Overview,
            Tab::Contributors,
            Tab::Files,
            Tab::Activity,
            Tab::Commits,
            Tab::Configurations,
            Tab::Overview
        ]
    );

    press(&mut app, &[KeyCode::BackTab]);
    assert_eq!(app.tab(), Tab::Configurations);
    press(&mut app, &[KeyCode::Left]);
    assert_eq!(app.tab(), Tab::Commits);
    press(&mut app, &[KeyCode::Char('h')]);
    assert_eq!(app.tab(), Tab::Activity);
    press(&mut app, &[KeyCode::Right, KeyCode::Char('l')]);
    assert_eq!(app.tab(), Tab::Configurations);
}

#[test]
fn digits_jump_straight_to_a_screen_and_six_is_the_configurations() {
    let mut app = app();
    for (digit, tab) in ['1', '2', '3', '4', '5', '6'].into_iter().zip(Tab::ALL) {
        press(&mut app, &[KeyCode::Char(digit)]);
        assert_eq!(app.tab(), tab);
    }
    assert_eq!(app.tab(), Tab::Configurations);
    press(&mut app, &[KeyCode::Char('7'), KeyCode::Char('0')]);
    assert_eq!(app.tab(), Tab::Configurations, "other digits do nothing");
}

#[test]
fn q_and_ctrl_c_quit_and_other_keys_do_not() {
    let mut app = app();
    press(
        &mut app,
        &[KeyCode::Char('x'), KeyCode::Enter, KeyCode::Esc],
    );
    assert!(!app.should_quit());
    press(&mut app, &[KeyCode::Char('q')]);
    assert!(app.should_quit());

    let mut app = self::app();
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.should_quit());
}

#[test]
fn ctrl_c_quits_even_while_typing_a_filter() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('/')]);
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.should_quit());
}

#[test]
fn help_opens_and_any_key_closes_it_without_acting() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('?')]);
    assert!(app.help_open());
    press(&mut app, &[KeyCode::Char('q')]);
    assert!(!app.help_open());
    assert!(!app.should_quit());
    press(&mut app, &[KeyCode::Char('?'), KeyCode::Tab]);
    assert_eq!(app.tab(), Tab::Overview);
}

#[test]
fn a_held_key_repeats() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('5')]);
    let mut repeat = key(KeyCode::Down);
    repeat.kind = KeyEventKind::Repeat;
    app.handle_key(repeat);
    app.handle_key(repeat);
    assert_eq!(app.commits.selected(), Some(2));
}

#[test]
fn key_releases_are_ignored() {
    let mut app = app();
    let mut release = key(KeyCode::Char('q'));
    release.kind = KeyEventKind::Release;
    app.handle_key(release);
    assert!(!app.should_quit());
}

#[test]
fn the_selection_moves_and_stops_at_the_ends() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('2')]);
    assert_eq!(app.contributors.selected(), Some(0));

    press(&mut app, &[KeyCode::Down]);
    assert_eq!(app.contributors.selected(), Some(1));
    press(
        &mut app,
        &[KeyCode::Down, KeyCode::Down, KeyCode::Char('j')],
    );
    assert_eq!(app.contributors.selected(), Some(1), "only 2 contributors");
    press(&mut app, &[KeyCode::Up]);
    assert_eq!(app.contributors.selected(), Some(0));
    press(&mut app, &[KeyCode::Char('k'), KeyCode::Up]);
    assert_eq!(app.contributors.selected(), Some(0));

    press(&mut app, &[KeyCode::End]);
    assert_eq!(app.contributors.selected(), Some(1));
    press(&mut app, &[KeyCode::Home]);
    assert_eq!(app.contributors.selected(), Some(0));
    press(&mut app, &[KeyCode::PageDown]);
    assert_eq!(app.contributors.selected(), Some(1));
    press(&mut app, &[KeyCode::PageUp]);
    assert_eq!(app.contributors.selected(), Some(0));
    press(&mut app, &[KeyCode::Char('G')]);
    assert_eq!(app.contributors.selected(), Some(1));
    press(&mut app, &[KeyCode::Char('g')]);
    assert_eq!(app.contributors.selected(), Some(0));
}

#[test]
fn each_screen_keeps_its_own_selection() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('2'), KeyCode::Down]);
    press(
        &mut app,
        &[KeyCode::Char('3'), KeyCode::Down, KeyCode::Down],
    );
    press(&mut app, &[KeyCode::Char('2')]);
    assert_eq!(app.contributors.selected(), Some(1));
    assert_eq!(app.files.selected(), Some(2));
}

#[test]
fn screens_without_a_list_ignore_movement() {
    let mut app = app();
    for tab in ['1', '4'] {
        press(&mut app, &[KeyCode::Char(tab), KeyCode::Down, KeyCode::End]);
    }
    assert_eq!(app.contributors.selected(), Some(0));
    assert_eq!(app.commits.selected(), Some(0));
}

#[test]
fn commits_are_listed_newest_first_and_the_selection_shows_details() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('5')]);
    assert_eq!(
        messages(&app),
        vec!["chore: many files", "fix: last", "msg", "feat: first"]
    );
    assert_eq!(app.selected_commit().unwrap().message, "chore: many files");
    press(&mut app, &[KeyCode::Down]);
    assert_eq!(app.selected_commit().unwrap().message, "fix: last");
}

#[test]
fn enter_on_a_contributor_opens_their_commits() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('2')]);
    assert_eq!(app.stats().contributors[0].name, "Alice");
    press(&mut app, &[KeyCode::Enter]);

    assert_eq!(app.tab(), Tab::Commits);
    assert_eq!(app.filter(), "alice@x.io");
    assert_eq!(messages(&app), vec!["fix: last", "feat: first"]);
    assert_eq!(app.visible_count(), 2);
    assert_eq!(app.total_commit_details(), 4);
    assert_eq!(app.selected_commit().unwrap().message, "fix: last");
}

#[test]
fn enter_does_nothing_outside_the_contributors_screen() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('3'), KeyCode::Enter]);
    assert_eq!(app.tab(), Tab::Files);
    assert_eq!(app.filter(), "");
}

#[test]
fn slash_starts_a_filter_that_narrows_the_list_as_you_type() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('/')]);
    assert_eq!(app.tab(), Tab::Commits, "/ jumps to the commit list");
    assert!(app.is_editing_filter());

    type_text(&mut app, "bob");
    assert_eq!(messages(&app), vec!["chore: many files", "msg"]);
    type_text(&mut app, "q2");
    assert_eq!(app.filter(), "bobq2");
    assert!(!app.should_quit());
    assert_eq!(app.tab(), Tab::Commits);
    assert_eq!(app.visible_count(), 0);
    assert_eq!(app.selected_commit(), None);

    press(&mut app, &[KeyCode::Backspace, KeyCode::Backspace]);
    assert_eq!(app.filter(), "bob");
    assert_eq!(app.visible_count(), 2);
    assert_eq!(app.selected_commit().unwrap().message, "chore: many files");
}

#[test]
fn enter_keeps_the_filter_and_stops_editing() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "last");
    press(&mut app, &[KeyCode::Enter]);

    assert!(!app.is_editing_filter());
    assert_eq!(app.filter(), "last");
    assert_eq!(messages(&app), vec!["fix: last"]);

    press(&mut app, &[KeyCode::Char('1')]);
    assert_eq!(app.tab(), Tab::Overview);
}

#[test]
fn esc_clears_the_filter_while_editing_and_afterwards() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "alice");
    press(&mut app, &[KeyCode::Esc]);
    assert!(!app.is_editing_filter());
    assert_eq!(app.filter(), "");
    assert_eq!(app.visible_count(), 4);

    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "alice");
    press(&mut app, &[KeyCode::Enter, KeyCode::Esc]);
    assert_eq!(app.filter(), "");
    assert_eq!(app.visible_count(), 4);
}

#[test]
fn the_filter_matches_author_email_message_and_hash_ignoring_case() {
    let mut app = app();
    let hash = app.stats().commit_details.as_ref().unwrap()[2].hash.clone();
    let prefix = hash[..8].to_uppercase();
    for (text, expected) in [
        ("ALICE", 2),
        ("bob@x", 2),
        ("FEAT", 1),
        ("many", 1),
        (prefix.as_str(), 1),
        ("nobody", 0),
    ] {
        press(&mut app, &[KeyCode::Char('/')]);
        type_text(&mut app, text);
        assert_eq!(app.visible_count(), expected, "filter {text:?}");
        press(&mut app, &[KeyCode::Esc]);
    }
}

#[test]
fn changing_the_filter_resets_the_selection_to_the_first_match() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('5'), KeyCode::End]);
    assert_eq!(app.commits.selected(), Some(3));
    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "a");
    assert_eq!(app.commits.selected(), Some(0));
}

#[test]
fn an_empty_repository_survives_every_key() {
    let mut app = empty_app();
    for code in [
        KeyCode::Tab,
        KeyCode::Down,
        KeyCode::End,
        KeyCode::Enter,
        KeyCode::Char('/'),
        KeyCode::Char('x'),
        KeyCode::Enter,
        KeyCode::Esc,
        KeyCode::Char('5'),
        KeyCode::PageDown,
        KeyCode::Char('2'),
        KeyCode::Enter,
    ] {
        app.handle_key(key(code));
    }
    assert_eq!(app.contributors.selected(), None);
    assert_eq!(app.commits.selected(), None);
    assert_eq!(app.selected_commit(), None);
    assert!(!app.should_quit());
}

#[test]
fn without_commit_details_the_list_is_simply_empty() {
    let mut stats = stats();
    stats.commit_details = None;
    let mut app = App::new(stats);
    press(
        &mut app,
        &[KeyCode::Char('5'), KeyCode::Down, KeyCode::Char('/')],
    );
    type_text(&mut app, "x");
    assert_eq!(app.visible_count(), 0);
    let text = screen(&mut app, 100, 24);
    assert!(text.contains("No commit selected"));
}

#[test]
fn the_overview_shows_the_repository_languages_and_contributors() {
    let mut app = app();
    let text = screen(&mut app, 110, 30);
    for expected in [
        "GITSCOPE v",
        "demo",
        "1 Overview",
        "5 Commits",
        "Repository",
        "Contributors",
        "Insertions",
        "+1,042",
        "Period",
        "2026-01-05 → 2026-03-08",
        "Languages",
        "Rust",
        "Top contributors",
        "Alice",
        "Bob",
        "? help",
        "q quit",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn the_filters_are_shown_in_the_title() {
    let mut stats = stats();
    stats.filters.author = Some("alice".into());
    let mut app = App::new(stats);
    let text = screen(&mut app, 110, 30);
    assert!(text.contains("author \"alice\""), "{text}");
}

#[test]
fn the_contributors_screen_lists_everyone_with_the_selection_marked() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('2')]);
    let text = screen(&mut app, 120, 20);
    for expected in [
        "Contributors (2)",
        "Enter: see their commits",
        "alice@x.io",
        "bob@x.io",
        "▶ Alice",
        "2026-01-05",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
    press(&mut app, &[KeyCode::Down]);
    assert!(screen(&mut app, 120, 20).contains("▶ Bob"));
}

#[test]
fn the_files_screen_lists_modified_files_and_extensions() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('3')]);
    let text = screen(&mut app, 110, 20);
    for expected in [
        "Most modified files",
        "a.rs",
        "Extensions",
        ".rs",
        "Total files",
        "3",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn the_activity_screen_shows_months_weekdays_and_hours() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('4')]);
    let text = screen(&mut app, 110, 24);
    for expected in [
        "Commits per month",
        "2026-01",
        "2026-03",
        "By weekday",
        "Mon",
        "Sun",
        "By hour (UTC)",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn the_commits_screen_shows_the_list_and_the_selected_commit() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('5'), KeyCode::Down]);
    let text = screen(&mut app, 120, 30);
    for expected in [
        "Commits (4)",
        "Date (UTC)",
        "chore: many files",
        "feat: first",
        "▶ 2026-03-07 14:50",
        "Details",
        "Alice <alice@x.io>",
        "Files changed",
        "+1,000",
        "M a.rs",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn the_details_summarise_files_that_do_not_fit() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('5')]);
    let text = screen(&mut app, 120, 20);
    assert!(text.contains("M dir/file00.rs"), "{text}");
    assert!(text.contains("more files"), "{text}");
    assert!(!text.contains("dir/file29.rs"), "{text}");
}

#[test]
fn a_narrow_terminal_stacks_the_list_above_the_details() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('5')]);
    let text = screen(&mut app, 80, 30);
    assert!(text.contains("Commits (4)"));
    assert!(text.contains("Details"));
}

#[test]
fn a_long_list_scrolls_to_keep_the_selection_visible() {
    let mut records = Vec::new();
    for i in 0..60 {
        let mut record = record_with("Alice", "alice@x.io", "2026-01-05 10:00", &["a.rs"], 1, 0);
        record.message = format!("commit number {i:02}");
        records.push(record);
    }
    let stats = analysis::analyze(
        info("many"),
        Filters::default(),
        records.into_iter().map(Ok),
        ["a.rs"].map(String::from),
        analysis::Options {
            commit_details: true,
        },
    )
    .unwrap();
    let mut app = App::new(stats);
    press(&mut app, &[KeyCode::Char('5')]);

    let top = screen(&mut app, 100, 14);
    assert!(top.contains("commit number 00"), "{top}");
    assert!(!top.contains("commit number 59"), "{top}");

    press(&mut app, &[KeyCode::End]);
    let bottom = screen(&mut app, 100, 14);
    assert!(bottom.contains("commit number 59"), "{bottom}");
    assert!(!bottom.contains("commit number 00"), "{bottom}");
}

#[test]
fn the_footer_shows_the_filter_being_typed() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "bob");
    let text = screen(&mut app, 100, 20);
    assert!(text.contains("/ bob█"), "{text}");
    assert!(text.contains("Enter: apply"), "{text}");
    assert!(text.contains("Commits (2 of 4) · filter: bob"), "{text}");

    press(&mut app, &[KeyCode::Enter]);
    let text = screen(&mut app, 100, 20);
    assert!(text.contains("Esc: clear filter"), "{text}");
}

#[test]
fn the_help_overlay_lists_the_keys() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('?')]);
    let text = screen(&mut app, 100, 24);
    for expected in [
        "Keys",
        "switch tab",
        "filter commits",
        "quit",
        "Press any key",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn an_empty_repository_says_so_on_every_screen() {
    let mut app = empty_app();
    for tab in ['1', '2', '3', '4', '5'] {
        press(&mut app, &[KeyCode::Char(tab)]);
        let text = screen(&mut app, 100, 20);
        assert!(
            text.contains("This repository has no commits yet."),
            "{text}"
        );
    }
    press(&mut app, &[KeyCode::Char('6')]);
    let text = screen(&mut app, 100, 20);
    assert!(!text.contains("no commits yet"), "{text}");
    assert!(text.contains("Language"), "{text}");

    let mut stats = stats();
    stats.commits.total = 0;
    stats.filters.author = Some("nobody".into());
    let text = screen(&mut App::new(stats), 100, 20);
    assert!(
        text.contains("No commits match the given filters."),
        "{text}"
    );
}

#[test]
fn drawing_never_panics_whatever_the_size() {
    for (width, height) in [
        (120, 40),
        (80, 24),
        (40, 10),
        (20, 5),
        (10, 3),
        (3, 3),
        (1, 1),
    ] {
        let mut app = app();
        for tab in ['1', '2', '3', '4', '5', '6'] {
            press(&mut app, &[KeyCode::Char(tab)]);
            screen(&mut app, width, height);
        }
        press(&mut app, &[KeyCode::Char('?')]);
        screen(&mut app, width, height);
        let mut empty = empty_app();
        screen(&mut empty, width, height);
    }
}

fn portuguese() -> App {
    App::with_config(
        stats(),
        Config {
            language: Language::Portuguese,
        },
        None,
    )
}

fn app_with_config_file() -> (App, tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gitscope").join("config.json");
    let app = App::with_config(stats(), Config::default(), Some(path.clone()));
    (app, dir, path)
}

#[test]
fn enter_and_space_switch_the_language_and_save_it() {
    let (mut app, _dir, path) = app_with_config_file();
    assert_eq!(app.language(), Language::English);
    assert_eq!(app.save_status(), &SaveStatus::Idle);

    press(&mut app, &[KeyCode::Char('6'), KeyCode::Enter]);
    assert_eq!(app.language(), Language::Portuguese);
    assert_eq!(app.save_status(), &SaveStatus::Saved);
    assert_eq!(Config::load(&path).unwrap().language, Language::Portuguese);

    press(&mut app, &[KeyCode::Char(' ')]);
    assert_eq!(app.language(), Language::English);
    assert_eq!(Config::load(&path).unwrap().language, Language::English);
}

#[test]
fn enter_and_space_do_nothing_to_the_language_on_other_screens() {
    let (mut app, _dir, path) = app_with_config_file();
    for tab in ['1', '3', '4', '5'] {
        press(
            &mut app,
            &[KeyCode::Char(tab), KeyCode::Enter, KeyCode::Char(' ')],
        );
    }
    assert_eq!(app.language(), Language::English);
    assert_eq!(app.save_status(), &SaveStatus::Idle);
    assert!(!path.exists(), "nothing was saved");
}

#[test]
fn the_saved_language_is_used_the_next_time() {
    let (mut app, _dir, path) = app_with_config_file();
    press(&mut app, &[KeyCode::Char('6'), KeyCode::Enter]);

    let next = App::with_config(stats(), Config::load(&path).unwrap(), Some(path));
    assert_eq!(next.language(), Language::Portuguese);
    assert_eq!(next.texts().tabs[0], "Visão geral");
}

#[test]
fn without_a_place_to_store_it_the_choice_lasts_the_session() {
    let mut app = App::with_config(stats(), Config::default(), None);
    press(&mut app, &[KeyCode::Char('6'), KeyCode::Enter]);
    assert_eq!(app.language(), Language::Portuguese);
    assert_eq!(app.save_status(), &SaveStatus::NotStored);
    assert!(screen(&mut app, 120, 20).contains("Não há onde guardar"));
}

#[test]
fn a_failed_save_is_reported_and_the_language_still_changes() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, "x").unwrap();
    let mut app = App::with_config(
        stats(),
        Config::default(),
        Some(blocker.join("config.json")),
    );

    press(&mut app, &[KeyCode::Char('6'), KeyCode::Enter]);

    assert_eq!(app.language(), Language::Portuguese);
    assert!(matches!(app.save_status(), SaveStatus::Failed(_)));
    assert!(screen(&mut app, 120, 20).contains("Não foi possível salvar"));
}

#[test]
fn space_is_text_while_typing_a_filter_and_only_closes_the_help() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "a b");
    assert_eq!(app.filter(), "a b");
    press(
        &mut app,
        &[KeyCode::Enter, KeyCode::Char('6'), KeyCode::Char('?')],
    );

    press(&mut app, &[KeyCode::Char(' ')]);
    assert!(!app.help_open());
    assert_eq!(
        app.language(),
        Language::English,
        "the space only closed the help"
    );
}

#[test]
fn the_configurations_screen_shows_the_options_and_the_current_one() {
    let mut app = app();
    press(&mut app, &[KeyCode::Char('6')]);
    let text = screen(&mut app, 110, 20);
    for expected in [
        "6 Configurations",
        "Setting",
        "Value",
        "▶ Language",
        "● English",
        "○ Português (Brasil)",
        "Enter or Space",
        "Enter/Space change",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn switching_the_language_translates_the_screen_at_once() {
    let (mut app, _dir, path) = app_with_config_file();
    press(&mut app, &[KeyCode::Char('6'), KeyCode::Enter]);
    let text = screen(&mut app, 120, 20);
    for expected in [
        "1 Visão geral",
        "6 Configurações",
        "Configuração",
        "Valor",
        "Idioma",
        "○ English",
        "● Português (Brasil)",
        "Enter ou Espaço",
        "Salvo em",
        "config.json",
        "Enter/Espaço mudar",
        "sair",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
    assert!(path.exists());
}

#[test]
fn every_screen_is_translated_and_leaves_no_english_behind() {
    let mut app = portuguese();
    let english = [
        "Overview",
        "Top contributors",
        "By weekday",
        "Most modified",
        "Press any key",
        "Files changed",
        "Commits per month",
        "No commit selected",
    ];

    let screens: [(char, &[&str]); 5] = [
        (
            '1',
            &[
                "Repositório",
                "Linguagens",
                "Principais contribuidores",
                "Período",
                "Inserções",
            ],
        ),
        (
            '2',
            &[
                "Contribuidores (2)",
                "E-mail",
                "Enter: ver os commits",
                "Primeiro",
                "Último",
            ],
        ),
        (
            '3',
            &[
                "Arquivos mais modificados",
                "Extensões",
                "Total de arquivos",
                "Caminho",
                "Mudanças",
            ],
        ),
        (
            '4',
            &[
                "Commits por mês",
                "Por dia da semana",
                "Por hora (UTC)",
                "Seg",
                "Dom",
            ],
        ),
        (
            '5',
            &[
                "Detalhes",
                "Autor",
                "Mensagem",
                "Arquivos alterados",
                "Data (UTC)",
                "… e mais",
            ],
        ),
    ];
    for (tab, expected) in screens {
        press(&mut app, &[KeyCode::Char(tab)]);
        let text = screen(&mut app, 130, 34);
        for word in expected {
            assert!(
                text.contains(word),
                "tab {tab}: missing {word:?} in:\n{text}"
            );
        }
        for word in english {
            assert!(
                !text.contains(word),
                "tab {tab}: English {word:?} left in:\n{text}"
            );
        }
    }
}

#[test]
fn the_filter_help_and_empty_texts_are_translated_too() {
    let mut app = portuguese();
    press(&mut app, &[KeyCode::Char('/')]);
    type_text(&mut app, "bob");
    let text = screen(&mut app, 120, 24);
    assert!(text.contains("Commits (2 de 4) · filtro: bob"), "{text}");
    assert!(text.contains("Enter: aplicar"), "{text}");
    press(&mut app, &[KeyCode::Enter]);
    assert!(screen(&mut app, 120, 24).contains("Esc: limpar filtro"));

    press(&mut app, &[KeyCode::Char('?')]);
    let text = screen(&mut app, 120, 24);
    for expected in [
        "Teclas",
        "trocar de aba",
        "em Configurações",
        "Pressione qualquer tecla",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }

    let mut filtered = stats();
    filtered.filters.author = Some("alice".into());
    filtered.commits.total = 0;
    let mut empty = App::with_config(
        filtered,
        Config {
            language: Language::Portuguese,
        },
        None,
    );
    let text = screen(&mut empty, 120, 20);
    assert!(
        text.contains("Nenhum commit casa com os filtros dados."),
        "{text}"
    );
    assert!(text.contains("autor \"alice\""), "{text}");
}

#[test]
fn wrap_breaks_at_words_and_respects_the_width() {
    use super::ui::wrap;
    assert_eq!(wrap("one two three", 20), vec!["one two three"]);
    assert_eq!(wrap("one two three", 7), vec!["one two", "three"]);
    assert_eq!(wrap("one two three", 3), vec!["one", "two", "thr", "ee"]);
    assert_eq!(wrap("a   b\n\nc", 10), vec!["a b c"]);
    assert_eq!(wrap("", 10), vec![""]);
    assert_eq!(wrap("   ", 10), vec![""]);
}

#[test]
fn wrap_splits_words_longer_than_the_line_and_always_advances() {
    use super::ui::wrap;
    assert_eq!(wrap("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
    assert_eq!(
        wrap("ab abcdefghij cd", 4),
        vec!["ab", "abcd", "efgh", "ij", "cd"]
    );
    assert_eq!(wrap("abc", 0), vec!["a", "b", "c"]);
}

#[test]
fn wrap_measures_screen_columns_not_bytes() {
    use super::ui::wrap;
    assert_eq!(wrap("ação ação ação", 9), vec!["ação ação", "ação"]);
    assert_eq!(wrap("日本語 日本語", 6), vec!["日本語", "日本語"]);
    assert_eq!(wrap("日本語日本語", 6), vec!["日本語", "日本語"]);
    assert_eq!(wrap("日日", 1), vec!["日", "日"]);
}

fn app_with_message(message: &str, paths: &[&str]) -> App {
    let mut record = record_with("Alice", "alice@x.io", "2026-03-07 14:50", paths, 3, 1);
    record.message = message.into();
    record.id = "cc33".repeat(10);
    let stats = analysis::analyze(
        info("demo"),
        Filters::default(),
        [record].into_iter().map(Ok),
        ["a.rs"].map(String::from),
        analysis::Options {
            commit_details: true,
        },
    )
    .unwrap();
    let mut app = App::new(stats);
    press(&mut app, &[KeyCode::Char('5')]);
    app
}

const LONG_MESSAGE: &str = "fix: handle the case where a repository has a very long commit subject that used to run off the edge of the screen END";

#[test]
fn a_long_message_wraps_inside_the_details_instead_of_running_off_the_screen() {
    for (width, height) in [(100, 24), (120, 30), (80, 30)] {
        let mut app = app_with_message(LONG_MESSAGE, &["a.rs"]);
        let text = screen(&mut app, width, height);

        for word in LONG_MESSAGE.split_whitespace() {
            assert!(
                text.contains(word),
                "{width}x{height}: {word:?} missing in:\n{text}"
            );
        }
        assert_eq!(
            text.matches("Message").count(),
            2,
            "one in the list header, one label"
        );
        let rows: Vec<&str> = text.lines().collect();
        let label = rows
            .iter()
            .position(|r| r.contains("Message") && r.contains("fix:"))
            .expect("the label row");
        let indent = format!("│{}", " ".repeat(20));
        let next = rows[label + 1];
        let at = next.rfind(&indent).unwrap_or_else(|| {
            panic!(
                "{width}x{height}: the continuation must be indented:
{text}"
            )
        });
        let after = next[at + indent.len()..].chars().next();
        assert!(
            after.is_some_and(|c| c != ' '),
            "{width}x{height}: text must start right after the indent:
{text}"
        );
    }
}

#[test]
fn a_long_path_is_broken_into_pieces_and_none_of_it_is_lost() {
    let long_path = "src/very/deeply/nested/directory/structure/that/never/seems/to/end/file_with_a_long_name.rs";
    let mut app = app_with_message("short", &[long_path]);
    let text = screen(&mut app, 60, 30);
    let rows: Vec<&str> = text.lines().collect();
    let details = rows.iter().position(|r| r.contains("Details")).unwrap();

    assert!(
        rows[details..].iter().any(|r| r.contains("M src/very")),
        "{text}"
    );
    let joined: String = rows[details + 1..]
        .iter()
        .flat_map(|r| r.chars())
        .filter(|c| !matches!(c, ' ' | '│' | '─' | '└' | '┘'))
        .collect();
    assert!(joined.contains(&format!("M{long_path}")), "{joined}");
}

#[test]
fn the_files_that_fit_are_counted_in_wrapped_lines() {
    let paths: Vec<String> = (0..12)
        .map(|i| format!("some/rather/long/directory/name/for/file{i:02}.rs"))
        .collect();
    let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
    let mut app = app_with_message("short", &paths);
    let text = screen(&mut app, 90, 22);
    assert!(text.contains("more files"), "{text}");
    assert!(text.contains("M some/rather"), "{text}");
    assert!(!text.contains("file11.rs"), "{text}");
}
