use std::ffi::OsStr;
use std::path::PathBuf;
use std::rc::Rc;

use times::Date;
use times::parse::from_stem;

pub struct Data {
    pub months: Vec<(Date, Rc<PathBuf>)>,
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

            months.push((date, path.into()));
        }
        months.sort_unstable_by_key(|(date, _)| *date);
        Ok(Self { months })
    }
}
