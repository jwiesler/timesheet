use crate::verify::{verify, Error};
use crate::{Day, Entry, Positioned, Time, Topic};

use std::fmt;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

pub struct Output<'a>(&'a [Day]);

impl<'a> Output<'a> {
    pub fn new(days: &'a [Day]) -> Result<Self, Error> {
        verify(days)?;
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

        let minutes: usize = self
            .entries
            .iter()
            .map(|e| &e.value)
            .tuple_windows()
            .map(|(entry, next)| {
                if let Topic::Project { .. } = &entry.topic {
                    next.time.elapsed_minutes(entry.time).unwrap()
                } else {
                    0
                }
            })
            .sum();
        let hours = minutes / 60;
        let minutes = minutes % 60;
        writeln!(f, "# Total: {hours:0>2}:{minutes:0>2}")?;
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
