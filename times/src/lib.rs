#![deny(rust_2018_idioms, nonstandard_style)]
#![warn(future_incompatible)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub};

pub use chrono::NaiveDate;
use chrono::{Datelike, Weekday};

pub mod convert;
pub mod format;
pub mod generate;
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

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct Date(NaiveDate);

impl Date {
    #[must_use]
    pub fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    #[must_use]
    pub fn today() -> Self {
        Self(chrono::offset::Local::now().date_naive())
    }

    #[must_use]
    pub fn year(&self) -> i32 {
        self.0.year()
    }

    #[must_use]
    pub fn month(&self) -> u32 {
        self.0.month()
    }

    #[must_use]
    pub fn day(&self) -> u32 {
        self.0.day()
    }

    #[must_use]
    pub fn is_weekday(&self) -> bool {
        !matches!(self.0.weekday(), Weekday::Sat | Weekday::Sun)
    }

    pub fn following_day_in_month(&self) -> Option<Self> {
        self.0
            .iter_days()
            .take_while(|d| d.month() == self.0.month())
            .map(Date)
            .nth(1)
    }

    pub fn next_weekday_in_month(&self) -> Option<Self> {
        self.0
            .iter_days()
            .take_while(|d| d.month() == self.0.month())
            .map(Date)
            .find(Date::is_weekday)
    }
}

fn weekday_to_str(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Mo",
        Weekday::Tue => "Di",
        Weekday::Wed => "Mi",
        Weekday::Thu => "Do",
        Weekday::Fri => "Fr",
        Weekday::Sat => "Sa",
        Weekday::Sun => "So",
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}. {}.{:0>2}.",
            weekday_to_str(self.0.weekday()),
            self.0.day(),
            self.0.month(),
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Entry {
    pub time: Time,
    pub topic: Topic,
}

#[derive(Debug)]
pub struct Day {
    pub comments: Vec<String>,
    pub date: Positioned<Date>,
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
