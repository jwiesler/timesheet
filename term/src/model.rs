use std::path::{Path, PathBuf};

use times::convert::{Error, Month};

pub struct Model {
    converted: Month,
    path: PathBuf,
}

impl Model {
    pub fn new(days: Vec<times::Day>, path: PathBuf) -> Result<Self, Error> {
        let month = Self::convert(days)?;
        Ok(Self {
            converted: month,
            path,
        })
    }

    fn convert(days: Vec<times::Day>) -> Result<Month, Error> {
        let converted = days
            .into_iter()
            .map(times::convert::Day::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Month::new(converted))
    }

    pub fn month(&self) -> &Month {
        &self.converted
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}
