use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{Context, anyhow};
use times::Date;
use times::convert::{Error, Month};
use times::parse::parse;

pub struct Model {
    path: Rc<PathBuf>,
    converted: Month,
    date: Date,
}

impl Model {
    pub fn load(date: Date, path: Rc<PathBuf>) -> anyhow::Result<Model> {
        let file = File::open(path.as_path())?;
        let days = parse(&mut BufReader::new(file), date)
            .with_context(|| anyhow!("Error trying to parse {path:?}"))?;
        let model = Model::new(date, days, path.clone())
            .with_context(move || anyhow!("Timesheets under {path:?} are invalid"))?;
        Ok(model)
    }

    pub fn new(date: Date, days: Vec<times::Day>, path: Rc<PathBuf>) -> Result<Self, Error> {
        let month = Self::convert(days)?;
        Ok(Self {
            converted: month,
            path,
            date,
        })
    }

    fn convert(days: Vec<times::Day>) -> Result<Month, Error> {
        let converted = days
            .into_iter()
            .map(times::convert::Day::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Month::new(converted))
    }

    pub fn path(&self) -> &Rc<PathBuf> {
        &self.path
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn month(&self) -> &Month {
        &self.converted
    }
}
