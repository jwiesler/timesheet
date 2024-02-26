use crate::{Day, Entry, Positioned, Time, Topic};
use itertools::Itertools;
use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Time span in line {0} is never terminated")]
    NotTerminated(usize),
}

pub struct Output<'a>(&'a [Day]);

impl<'a> Output<'a> {
    pub fn new(days: &'a [Day]) -> Result<Self, Error> {
        for day in days {
            if let Some(last) = day.entries.last() {
                if !matches!(last.value.topic, Topic::Break) {
                    return Err(Error::NotTerminated(last.line));
                }
            }
        }
        Ok(Self(days))
    }
}

trait Format {
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<'a> Display for Output<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.format(f)
    }
}

impl<'a> Format for &'a [Day] {
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "* {}", self.day.value)?;
        self.entries.as_slice().format(f)?;
        Ok(())
    }
}

impl<'a> Format for &'a [Positioned<Entry>] {
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (entry, next) in self.iter().map(|e| &e.value).tuple_windows() {
            (entry, next).format(f)?;
        }
        Ok(())
    }
}

impl<'a> Format for (&'a Entry, &'a Entry) {
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Topic::Project {
            identifier,
            comment,
        } = &self.0.topic
        {
            write!(f, "{} - {} {}", self.0.time, self.1.time, identifier)?;
            if let Some(comment) = comment {
                write!(f, " {comment}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.hour, self.minute)
    }
}
