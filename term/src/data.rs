use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use times::Date;
use times::parse::{from_stem, parse};

use crate::model::Model;

pub struct Data {
    pub months: Vec<(Date, PathBuf)>,
}

impl Data {
    pub fn from_dir(path: PathBuf) -> std::io::Result<Self> {
        let mut months = Vec::new();
        for file in path.read_dir()? {
            let file = file?;
            if !file.file_type()?.is_file() {
                continue;
            }
            let path = file.path();
            if path.extension() != Some(OsStr::new("tsh")) {
                continue;
            }

            let stem = path.file_stem().unwrap().to_str().unwrap();
            let date = from_stem(stem).unwrap_or_else(|| {
                panic!(
                    "failed to parse month from input file stem {stem:?}, expected format YYYY-MM"
                )
            });

            months.push((date, path));
        }
        months.sort_unstable_by_key(|(date, _)| *date);
        Ok(Self { months })
    }

    pub fn load_month(date: Date, path: &Path) -> std::io::Result<Model> {
        let file = File::open(path)?;
        let days = parse(&mut BufReader::new(file), date)
            .map_err(|e| std::io::Error::other(format!("Error trying to read {path:?}: {e}")))?;
        let model = Model::new(days, path.to_path_buf()).map_err(|e| {
            std::io::Error::other(format!("Timesheets under {path:?} are invalid: {e}"))
        })?;
        Ok(model)
    }
}
