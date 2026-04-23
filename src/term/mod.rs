mod app;
mod components;
mod data;
mod editor;
mod model;
mod style;

use std::io;
use std::path::Path;

use anyhow::{Context, anyhow};
use app::App;
use components::month::Month;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use times::Date;

use crate::term::data::Data;
use crate::term::model::Model;

struct Terminal(DefaultTerminal);

impl Terminal {
    fn new() -> io::Result<Terminal> {
        let terminal = ratatui::init();
        let mut res = Terminal(terminal);
        execute!(res.0.backend_mut(), EnableMouseCapture)?;
        Ok(res)
    }

    fn try_restore(&mut self) -> io::Result<()> {
        ratatui::try_restore()?;
        execute!(self.0.backend_mut(), DisableMouseCapture)?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if let Err(err) = self.try_restore() {
            // There's not much we can do if restoring the terminal fails, so we just print the error
            eprintln!("Failed to restore terminal: {err}");
        }
    }
}

pub fn run_term(path: &Path) -> anyhow::Result<()> {
    let dir = path.parent().unwrap();
    let state = Data::from_dir(dir).context("Failed to collect timesheets")?;
    let today = Date::today();
    let month = {
        let (date, path) = state
            .months
            .iter()
            .find(|(_, p)| p.as_path() == path)
            .or(state.months.last())
            .with_context(|| anyhow!("No timesheets were found under {dir:?}"))?
            .clone();
        let month = Model::load(date, path.clone())?;
        Month::new(month)
    };
    let mut terminal = Terminal::new()?;
    App::new(state, today, month).run(&mut terminal.0)
}
