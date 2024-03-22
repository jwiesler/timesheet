use crate::convert::{AccumulatedTime, Day};
use crate::Minutes;

use std::fmt::{Display, Formatter, Result};
use std::ops::Add;

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

fn output_time_delta(f: &mut Formatter, lhs: Minutes, rhs: Minutes) -> Result {
    if lhs < rhs {
        let delta = rhs - lhs;
        let duration = delta.into_duration();
        write!(f, "-{duration}")
    } else {
        let delta = lhs - rhs;
        let duration = delta.into_duration();
        write!(f, "+{duration}")
    }
}

impl<'a> Format for &'a [Day] {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        let mut first = true;
        let mut expected_min_work = Minutes::default();
        for day in *self {
            if day.entries.is_empty() {
                continue;
            }
            expected_min_work += Minutes::from_hours(8);
            if first {
                first = false;
            } else {
                writeln!(f)?;
            }
            day.format(f)?;
        }

        let time = self
            .iter()
            .map(|d| d.times.clone())
            .fold(AccumulatedTime::default(), AccumulatedTime::add);
        let minutes = time.billable_time();
        let duration = minutes.into_duration();
        writeln!(f)?;
        write!(f, "# Total {duration} (")?;
        output_time_delta(f, minutes, expected_min_work)?;
        writeln!(f, ")")?;

        Ok(())
    }
}

impl<'a> Format for &'a Day {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "* {}", self.day.value)?;
        let minutes = self.times.billable_time();
        if minutes != Minutes::default() {
            let duration = minutes.into_duration();
            write!(f, " -> {duration}")?;
            let expected_time = Minutes::from_hours(8);
            if minutes == expected_time {
                writeln!(f)?;
            } else {
                write!(f, " (")?;
                output_time_delta(f, minutes, expected_time)?;
                writeln!(f, ")")?;
            }
        } else {
            writeln!(f)?;
        }

        crate::format::Format::format(self.entries.as_slice(), f)?;

        Ok(())
    }
}
