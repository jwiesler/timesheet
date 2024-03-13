use crate::convert::{Day, Entry};
use crate::{Positioned, Time};

use std::fmt::{Display, Formatter, Result};

pub struct Output<'a>(pub &'a [Day]);

pub trait Format {
    fn format(&self, f: &mut Formatter<'_>) -> Result;
}

impl<'a> Display for Output<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.format(f)
    }
}

impl<'a> Format for &'a [Day] {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
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
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "* {}", self.day.value)?;
        self.entries.as_slice().format(f)?;

        Ok(())
    }
}

impl Format for [Positioned<Entry>] {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        for entry in self {
            entry.value.format(f)?;
        }
        Ok(())
    }
}

impl Format for Entry {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} - {} {}",
            self.start.value, self.end.value, self.identifier
        )?;
        if let Some(comment) = &self.comment {
            write!(f, " {comment}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:0>2}:{:0>2}", self.hour, self.minute)
    }
}
