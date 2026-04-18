use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, anyhow};
use times::Date;
use times::parse::from_stem;

pub struct Data {
    pub months: Vec<(Date, Rc<PathBuf>)>,
}

impl Data {
    pub fn from_dir(path: &Path) -> anyhow::Result<Self> {
        let mut months = Vec::new();
        for file in path
            .read_dir()
            .with_context(|| anyhow!("Failed to read dir {path:?}"))?
        {
            let file = file?;
            if !file.file_type()?.is_file() {
                continue;
            }
            let path = file.path();
            if path.extension() != Some(OsStr::new("tsh")) {
                continue;
            }

            let stem = path.file_stem().unwrap().to_str().unwrap();
            let date = from_stem(stem).with_context(|| {
                anyhow!(
                    "failed to parse month from input file {path:?}, expected format YYYY-MM.tsh"
                )
            })?;

            months.push((date, path.into()));
        }
        months.sort_unstable_by_key(|(date, _)| *date);
        Ok(Self { months })
    }
}
