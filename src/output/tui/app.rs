use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::TableState;

use super::i18n::{self, Texts};
use crate::config::{Config, Language};
use crate::model::{CommitInfo, RepositoryStats};

const PAGE: isize = 10;
const SETTINGS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Contributors,
    Files,
    Activity,
    Commits,
    Configurations,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Overview,
        Tab::Contributors,
        Tab::Files,
        Tab::Activity,
        Tab::Commits,
        Tab::Configurations,
    ];

    pub fn index(self) -> usize {
        Tab::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    fn next(self) -> Tab {
        Tab::ALL[(self.index() + 1) % Tab::ALL.len()]
    }

    fn previous(self) -> Tab {
        Tab::ALL[(self.index() + Tab::ALL.len() - 1) % Tab::ALL.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SaveStatus {
    #[default]
    Idle,
    Saved,
    NotStored,
    Failed(String),
}

pub struct App {
    pub(super) stats: RepositoryStats,
    tab: Tab,
    help: bool,
    quit: bool,
    pub(super) contributors: TableState,
    pub(super) files: TableState,
    pub(super) commits: TableState,
    filter: String,
    editing_filter: bool,
    visible: Vec<usize>,
    language: Language,
    config_path: Option<PathBuf>,
    save_status: SaveStatus,
    pub(super) settings: TableState,
}

impl App {
    pub fn new(stats: RepositoryStats) -> Self {
        Self::with_config(stats, Config::default(), None)
    }

    pub fn with_config(
        stats: RepositoryStats,
        config: Config,
        config_path: Option<PathBuf>,
    ) -> Self {
        let mut app = Self {
            tab: Tab::Overview,
            help: false,
            quit: false,
            contributors: TableState::default(),
            files: TableState::default(),
            commits: TableState::default(),
            filter: String::new(),
            editing_filter: false,
            visible: Vec::new(),
            language: config.language,
            config_path,
            save_status: SaveStatus::Idle,
            settings: TableState::default(),
            stats,
        };
        app.settings.select(Some(0));
        app.contributors
            .select((!app.stats.contributors.is_empty()).then_some(0));
        app.files
            .select((!app.stats.files.most_modified.is_empty()).then_some(0));
        app.refilter();
        app
    }

    pub fn texts(&self) -> &'static Texts {
        i18n::texts(self.language)
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn save_status(&self) -> &SaveStatus {
        &self.save_status
    }

    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    pub fn stats(&self) -> &RepositoryStats {
        &self.stats
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn help_open(&self) -> bool {
        self.help
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn is_editing_filter(&self) -> bool {
        self.editing_filter
    }

    pub fn visible_commits(&self) -> impl Iterator<Item = &CommitInfo> {
        let details = self.details();
        self.visible.iter().map(move |&i| &details[i])
    }

    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    pub fn total_commit_details(&self) -> usize {
        self.details().len()
    }

    pub fn selected_commit(&self) -> Option<&CommitInfo> {
        let row = self.commits.selected()?;
        self.details().get(*self.visible.get(row)?)
    }

    fn details(&self) -> &[CommitInfo] {
        self.stats.commit_details.as_deref().unwrap_or(&[])
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // O Windows envia também o evento de soltar a tecla; a repetição (segurar
        // a seta) chega como `Repeat` em alguns terminais e deve valer.
        if key.kind == KeyEventKind::Release {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.editing_filter {
            self.edit_filter(key.code);
            return;
        }
        if self.help {
            self.help = false;
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.help = true,
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.tab = self.tab.next(),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.tab = self.tab.previous();
            }
            KeyCode::Char(digit @ '1'..='6') => {
                self.tab = Tab::ALL[digit as usize - '1' as usize];
            }
            KeyCode::Down | KeyCode::Char('j') => self.step(1),
            KeyCode::Up | KeyCode::Char('k') => self.step(-1),
            KeyCode::PageDown => self.step(PAGE),
            KeyCode::PageUp => self.step(-PAGE),
            KeyCode::Home | KeyCode::Char('g') => self.step(isize::MIN),
            KeyCode::End | KeyCode::Char('G') => self.step(isize::MAX),
            KeyCode::Char('/') => {
                self.tab = Tab::Commits;
                self.editing_filter = true;
            }
            KeyCode::Esc => self.set_filter(String::new()),
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(),
            _ => {}
        }
    }

    fn edit_filter(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                let mut filter = std::mem::take(&mut self.filter);
                filter.push(c);
                self.set_filter(filter);
            }
            KeyCode::Backspace => {
                let mut filter = std::mem::take(&mut self.filter);
                filter.pop();
                self.set_filter(filter);
            }
            KeyCode::Enter => self.editing_filter = false,
            KeyCode::Esc => {
                self.editing_filter = false;
                self.set_filter(String::new());
            }
            _ => {}
        }
    }

    fn activate(&mut self) {
        match self.tab {
            Tab::Contributors => self.open_selected_contributor(),
            Tab::Configurations => self.change_setting(),
            _ => {}
        }
    }

    fn change_setting(&mut self) {
        self.language = self.language.next();
        self.persist();
    }

    fn persist(&mut self) {
        let config = Config {
            language: self.language,
        };
        self.save_status = match &self.config_path {
            None => SaveStatus::NotStored,
            Some(path) => match config.save(path) {
                Ok(()) => SaveStatus::Saved,
                Err(err) => SaveStatus::Failed(err.to_string()),
            },
        };
    }

    fn open_selected_contributor(&mut self) {
        let email = self
            .contributors
            .selected()
            .and_then(|i| self.stats.contributors.get(i))
            .map(|c| c.email.clone());
        if let Some(email) = email {
            self.set_filter(email);
            self.tab = Tab::Commits;
        }
    }

    fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.refilter();
    }

    fn refilter(&mut self) {
        let needle = self.filter.to_lowercase();
        self.visible = self
            .details()
            .iter()
            .enumerate()
            .filter(|(_, commit)| needle.is_empty() || matches(commit, &needle))
            .map(|(index, _)| index)
            .collect();
        self.commits.select((!self.visible.is_empty()).then_some(0));
        *self.commits.offset_mut() = 0;
    }

    fn step(&mut self, delta: isize) {
        let (state, len) = match self.tab {
            Tab::Contributors => (&mut self.contributors, self.stats.contributors.len()),
            Tab::Files => (&mut self.files, self.stats.files.most_modified.len()),
            Tab::Commits => (&mut self.commits, self.visible.len()),
            Tab::Configurations => (&mut self.settings, SETTINGS),
            Tab::Overview | Tab::Activity => return,
        };
        if len == 0 {
            state.select(None);
            return;
        }
        let current = state.selected().unwrap_or(0) as isize;
        let next = current.saturating_add(delta).clamp(0, len as isize - 1);
        state.select(Some(next as usize));
    }
}

fn matches(commit: &CommitInfo, needle: &str) -> bool {
    commit.author.to_lowercase().contains(needle)
        || commit.email.to_lowercase().contains(needle)
        || commit.message.to_lowercase().contains(needle)
        || commit.hash.contains(needle)
}
