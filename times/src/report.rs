use crate::convert::Day;
use crate::Minutes;

use std::fmt::{Display, Formatter, Result};

pub struct Output<'a>(pub &'a [Day]);

impl<'a> Output<'a> {
    #[must_use]
    pub fn new(days: &'a [Day]) -> Self {
        Self(days)
    }
}

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
            if day.entries.is_empty() {
                continue;
            }
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
        crate::format::Format::format(self, f)?;

        let time = self.accumulated_time();
        let minutes = time.billable_travel_time() + time.work_time();
        if minutes != Minutes::default() {
            let (hours, minutes) = minutes.hours_minutes();
            writeln!(f, "# Total: {hours:0>2}:{minutes:0>2}")?;
        }

        Ok(())
    }
}
