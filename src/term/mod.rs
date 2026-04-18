mod app;
mod components;
mod data;
mod editor;
mod model;
mod style;

use std::path::Path;

use anyhow::Context;
use app::App;
use components::month::Month;
use times::Date;

use crate::term::data::Data;
use crate::term::model::Model;

pub fn run_term(path: &Path) -> anyhow::Result<()> {
    let state = Data::from_dir(path.parent().unwrap()).context("Failed to collect timesheets")?;
    let today = Date::today();
    let month = {
        let (date, path) = state
            .months
            .iter()
            .find(|(_, p)| p.as_path() == path)
            .or(state.months.last())
            .unwrap()
            .clone();
        let month = Model::load(date, path.clone())?;
        Month::new(month)
    };
    let mut terminal = ratatui::init();
    let result = App::new(state, today, month).run(&mut terminal);
    ratatui::restore();
    result
}
