#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub};

pub mod convert;
pub mod format;
pub mod parse;
pub mod report;

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct Minutes(usize);

impl Minutes {
    #[must_use]
    pub fn from_hours(hours: usize) -> Self {
        Self(hours.checked_mul(60).unwrap())
    }

    #[must_use]
    pub fn into_duration(self) -> ClockDuration {
        ClockDuration {
            hours: self.0 / 60,
            minutes: self.0 % 60,
        }
    }

    #[must_use]
    pub fn into_inner(self) -> usize {
        self.0
    }
}

impl AddAssign<Minutes> for Minutes {
    fn add_assign(&mut self, rhs: Minutes) {
        self.0 += rhs.0;
    }
}

impl Sub<Minutes> for Minutes {
    type Output = Self;

    fn sub(self, rhs: Minutes) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

pub struct ClockDuration {
    hours: usize,
    minutes: usize,
}

impl Display for ClockDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.hours, self.minutes)
    }
}

impl Add<Minutes> for Minutes {
    type Output = Minutes;

    fn add(self, rhs: Minutes) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Sum<Minutes> for Minutes {
    fn sum<I: Iterator<Item = Minutes>>(iter: I) -> Self {
        iter.fold(Minutes::default(), |a, b| a + b)
    }
}

impl From<usize> for Minutes {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

impl Time {
    #[must_use]
    pub fn new(hour: u8, minute: u8) -> Option<Self> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) {
            None
        } else {
            Some(Self { hour, minute })
        }
    }

    #[must_use]
    pub fn elapsed(self, o: Time) -> Option<Minutes> {
        let hours = self.hour.checked_sub(o.hour)?;
        let hour_minutes = usize::from(hours) * 60;
        let minutes = if self.minute >= o.minute {
            hour_minutes + usize::from(self.minute.checked_sub(o.minute).unwrap())
        } else {
            hour_minutes.checked_sub(usize::from(o.minute.checked_sub(self.minute).unwrap()))?
        };
        Some(minutes.into())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Topic {
    Break,
    Project {
        identifier: String,
        comment: Option<String>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct Entry {
    pub time: Time,
    pub topic: Topic,
}

#[derive(Debug)]
pub struct Day {
    pub comments: Vec<String>,
    pub day: Positioned<String>,
    pub entries: Vec<Positioned<Entry>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Positioned<T> {
    pub line: usize,
    pub value: T,
}

impl<T> Positioned<T> {
    pub fn new(line: usize, value: T) -> Positioned<T> {
        Self { line, value }
    }
}

#[cfg(test)]
mod test {
    use crate::Time;

    #[test]
    fn test_elapsed() {
        assert_eq!(
            Time::new(12, 00)
                .unwrap()
                .elapsed(Time::new(11, 00).unwrap()),
            Some(60.into())
        );
        assert_eq!(
            Time::new(12, 10)
                .unwrap()
                .elapsed(Time::new(11, 5).unwrap()),
            Some(65.into())
        );
        assert_eq!(
            Time::new(12, 5)
                .unwrap()
                .elapsed(Time::new(11, 10).unwrap()),
            Some(55.into())
        );
        assert_eq!(
            Time::new(11, 00)
                .unwrap()
                .elapsed(Time::new(12, 00).unwrap()),
            None
        );
        assert_eq!(
            Time::new(12, 00)
                .unwrap()
                .elapsed(Time::new(12, 1).unwrap()),
            None
        );
    }
}
