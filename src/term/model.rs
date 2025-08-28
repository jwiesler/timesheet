use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;

use times::Date;
use times::convert::{Error, Month};
use times::parse::parse;

pub struct Model {
    path: Rc<PathBuf>,
    converted: Month,
    date: Date,
}

impl Model {
    pub fn load(date: Date, path: Rc<PathBuf>) -> std::io::Result<Model> {
        let file = File::open(path.as_path())?;
        let days = parse(&mut BufReader::new(file), date)
            .map_err(|e| std::io::Error::other(format!("Error trying to read {path:?}: {e}")))?;
        let model = Model::new(date, days, path.clone()).map_err(move |e| {
            std::io::Error::other(format!("Timesheets under {path:?} are invalid: {e}"))
        })?;
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
