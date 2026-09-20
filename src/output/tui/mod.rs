mod app;
mod i18n;
mod ui;

#[cfg(test)]
mod tests;

use std::io;
use std::path::PathBuf;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event};

pub use app::{App, SaveStatus, Tab};

use crate::config::Config;
use crate::model::RepositoryStats;

pub fn run(stats: RepositoryStats, config: Config, config_path: Option<PathBuf>) -> io::Result<()> {
    let mut app = App::with_config(stats, config, config_path);
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit() {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }
    Ok(())
}
