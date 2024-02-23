use crate::{Day, Entry, Positioned, Time, Topic};
use itertools::Itertools;
use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Format error: {0}")]
    Fmt(#[from] fmt::Error),
    #[error("Time in line {0} is never terminated")]
    NotTerminated(usize),
}

pub struct Output<'a>(pub &'a [Day]);

pub trait Format {
    fn format(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}

impl<'a> Display for Output<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.format(f).map_err(|_| fmt::Error)
    }
}

impl<'a> Format for &'a [Day] {
    fn format(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut first = true;
        for day in *self {
            if first {
                first = false;
            } else {
                writeln!(f)?;
            }
            day.format(f)?;
        }

        Ok(())
    }
}

impl<'a> Format for &'a Day {
    fn format(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "* {}", self.day.value)?;
        self.entries.as_slice().format(f)?;
        Ok(())
    }
}

impl<'a> Format for &'a [Positioned<Entry>] {
    fn format(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(last) = self.last() {
            if !matches!(last.value.topic, Topic::Break) {
                return Err(Error::NotTerminated(last.line));
            }
        }
        for (entry, next) in self.iter().map(|e| &e.value).tuple_windows() {
            if let Topic::Project {
                identifier,
                comment,
            } = &entry.topic
            {
                write!(f, "{} - {} {}", entry.time, next.time, identifier)?;
                if let Some(comment) = comment {
                    write!(f, " {comment}")?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.hour, self.minute)
    }
}
