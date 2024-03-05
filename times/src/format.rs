use crate::convert::{Day, Entry};
use crate::{Minutes, Positioned, Time};

use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Output<'a>(&'a [Day]);

impl<'a> Output<'a> {
    #[must_use]
    pub fn new(days: &'a [Day]) -> Self {
        Self(days)
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
        let minutes: Minutes = self.entries.iter().map(|e| e.value.duration).sum();
        let (hours, minutes) = minutes.hours_minutes();
        writeln!(f, "# Total: {hours:0>2}:{minutes:0>2}")?;
        Ok(())
    }
}

impl Format for [Positioned<Entry>] {
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for entry in self {
            entry.value.format(f)?;
        }
        Ok(())
    }
}

impl Format for Entry {
    fn format(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.hour, self.minute)
    }
}
