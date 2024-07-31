use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::mem::take;
use std::str::FromStr;

use chrono::format::{Item, Numeric, Pad, Parsed};
use chrono::{Datelike, Weekday};
use thiserror::Error;

use crate::{Date, Day, Entry, Positioned, Time, Topic};

#[derive(Debug, Error, Eq, PartialEq)]
pub enum EntryError {
    #[error("Invalid time format")]
    Time,
    #[error("Missing time")]
    MissingTime,
    #[error("Failed to parse date of day: {0}")]
    Date(DateError),
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DateError {
    #[error("Expected a date in the format <day of week>. <day>.<month>.")]
    Format,
    #[error("Invalid date")]
    Date,
    #[error("Invalid day of week")]
    DayOfWeek,
    #[error("Day of week does not match the given date")]
    UnexpectedDayOfWeek,
    #[error("Month does not match the given month")]
    UnexpectedMonth,
    #[error("Entry out of order, expected strictly monotonically increasing dates")]
    EntryOutOfOrder,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Many(EntryErrors),
    #[error("Expected a day in line {0}")]
    ExpectedDay(usize),
}

#[derive(Debug)]
pub struct EntryErrors(pub Vec<Positioned<EntryError>>);

impl Display for EntryErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered the following errors while parsing:")?;
        for Positioned { line, value } in &self.0 {
            write!(f, "\nFailed to parse entry in line {line}: {value}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeError;

impl FromStr for Time {
    type Err = TimeError;

    fn from_str(s: &str) -> Result<Self, TimeError> {
        let (hour, minute) = s.split_once(':').ok_or(TimeError)?;
        if hour.len() != 2 || minute.len() != 2 {
            return Err(TimeError);
        }
        Ok(Time {
            hour: hour.parse().map_err(|_| TimeError)?,
            minute: minute.parse().map_err(|_| TimeError)?,
        })
    }
}

impl FromStr for Topic {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug_assert!(s.trim() == s);

        if s.is_empty() {
            Ok(Topic::Break)
        } else if let Some((identifier, rest)) = s.split_once(|c: char| c.is_whitespace()) {
            Ok(Topic::Project {
                identifier: identifier.to_string(),
                comment: Some(rest.trim_start().to_owned()),
            })
        } else {
            Ok(Topic::Project {
                identifier: s.to_string(),
                comment: None,
            })
        }
    }
}

impl FromStr for Entry {
    type Err = EntryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug_assert!(s.trim() == s);
        if s.is_empty() {
            Err(EntryError::MissingTime)
        } else {
            let (time, rest) = if let Some((time, rest)) = s.split_once(|c: char| c.is_whitespace())
            {
                (time, rest)
            } else {
                (s, "")
            };
            let time = time.parse().map_err(|_| EntryError::Time)?;
            let topic = rest.trim_start().parse().unwrap();
            Ok(Entry { time, topic })
        }
    }
}

fn parse_weekday(s: &str) -> Result<Weekday, DateError> {
    match s {
        "Mo" => Ok(Weekday::Mon),
        "Di" => Ok(Weekday::Tue),
        "Mi" => Ok(Weekday::Wed),
        "Do" => Ok(Weekday::Thu),
        "Fr" => Ok(Weekday::Fri),
        "Sa" => Ok(Weekday::Sat),
        "So" => Ok(Weekday::Sun),
        _ => Err(DateError::DayOfWeek),
    }
}

fn parse_date(line: &str, month: Date, after: u32) -> Result<Date, DateError> {
    const ITEMS: &[Item<'static>] = &[
        Item::Numeric(Numeric::Day, Pad::Zero),
        Item::Literal("."),
        Item::Numeric(Numeric::Month, Pad::Zero),
        Item::Literal("."),
    ];
    let (weekday, date) = line.split_once('.').ok_or(DateError::Format)?;

    let mut parsed = Parsed::new();
    chrono::format::parse(&mut parsed, date.trim(), ITEMS.iter()).map_err(|_| DateError::Format)?;
    parsed.set_year(month.year().into()).unwrap();
    let date = parsed.to_naive_date().map_err(|_| DateError::Date)?;
    if date.month() != month.month() {
        return Err(DateError::UnexpectedMonth);
    }

    let weekday = parse_weekday(weekday.trim())?;
    if date.weekday() != weekday {
        return Err(DateError::UnexpectedDayOfWeek);
    }

    if date.day() <= after {
        return Err(DateError::EntryOutOfOrder);
    }

    Ok(Date(date))
}

pub fn parse(r: impl BufRead, month: Date) -> Result<Vec<Day>, Error> {
    let mut days = Vec::new();
    let mut current_day: Option<Day> = None;
    let mut comments = Vec::new();
    let mut errors = Vec::new();
    for (index, line) in r.lines().enumerate() {
        let index = index + 1;
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(comment) = line.strip_prefix('#') {
            comments.push(comment.to_owned());
        } else if let Some(line) = line.strip_prefix('*') {
            let last_day = current_day.take().map(|day| {
                let date = day.date.value.0.day();
                days.push(day);
                date
            });

            let date = parse_date(line, month, last_day.unwrap_or_default()).unwrap_or_else(|e| {
                errors.push(Positioned::new(index, EntryError::Date(e)));
                month
            });
            current_day = Some(Day {
                comments: take(&mut comments),
                date: Positioned::new(index, date),
                entries: Vec::new(),
            });
        } else {
            let day = current_day
                .as_mut()
                .ok_or_else(|| Error::ExpectedDay(index))?;
            match line.parse() {
                Ok(entry) => {
                    day.entries.push(Positioned::new(index, entry));
                }
                Err(e) => errors.push(Positioned::new(index, e)),
            }
        }
    }
    if let Some(day) = current_day.take() {
        days.push(day);
    }
    if errors.is_empty() {
        Ok(days)
    } else {
        Err(Error::Many(EntryErrors(errors)))
    }
}

#[must_use]
pub fn from_stem(stem: &str) -> Option<Date> {
    const ITEMS: &[Item<'static>] = &[
        Item::Numeric(Numeric::Year, Pad::Zero),
        Item::Literal("-"),
        Item::Numeric(Numeric::Month, Pad::Zero),
    ];

    let mut parsed = Parsed::new();
    chrono::format::parse(&mut parsed, stem, ITEMS.iter()).ok()?;
    parsed.set_day(1).ok()?;
    parsed.to_naive_date().ok().map(Date)
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use crate::parse::{parse_date, DateError, EntryError, TimeError};
    use crate::{Date, Entry, Time, Topic};

    #[test]
    fn test_parse_date() {
        let month = Date(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());

        assert_eq!(
            parse_date("Sa. 20.04.", month, 0),
            Ok(Date(NaiveDate::from_ymd_opt(2024, 4, 20).unwrap()))
        );

        assert_eq!(
            parse_date("Sa. 20.04.", month, 20),
            Err(DateError::EntryOutOfOrder)
        );

        let tests = [
            ("", DateError::Format),
            ("20.04.", DateError::Format),
            ("Sa 20.04.", DateError::Format),
            ("Si. 20.04.", DateError::DayOfWeek),
            ("Sa. 20.04", DateError::Format),
            ("Sa. 31.04.", DateError::Date),
            ("So. 20.04.", DateError::UnexpectedDayOfWeek),
            ("Sa. 20.05.", DateError::UnexpectedMonth),
        ];

        for (text, e) in tests {
            assert_eq!(parse_date(text, month, 0), Err(e), "{text}");
        }
    }

    #[test]
    fn test_parse_time() {
        assert_eq!("10:02".parse(), Ok(Time::new(10, 2).unwrap()));
        assert_eq!("10:20".parse(), Ok(Time::new(10, 20).unwrap()));
        assert_eq!("00:20".parse(), Ok(Time::new(0, 20).unwrap()));
        assert_eq!("10:2".parse::<Time>(), Err(TimeError));
        assert_eq!("1:20".parse::<Time>(), Err(TimeError));
        assert_eq!("10".parse::<Time>(), Err(TimeError));
        assert_eq!(":10".parse::<Time>(), Err(TimeError));
        assert_eq!("10:".parse::<Time>(), Err(TimeError));
        assert_eq!("".parse::<Time>(), Err(TimeError));
    }

    #[test]
    fn test_parse_topic() {
        assert_eq!(
            "Test".parse(),
            Ok(Topic::Project {
                identifier: "Test".to_owned(),
                comment: None,
            })
        );
        assert_eq!(
            "Test    bla".parse(),
            Ok(Topic::Project {
                identifier: "Test".to_owned(),
                comment: Some("bla".to_owned()),
            })
        );
        assert_eq!(
            "Test bla bla bla".parse(),
            Ok(Topic::Project {
                identifier: "Test".to_owned(),
                comment: Some("bla bla bla".to_owned()),
            })
        );
        assert_eq!("".parse(), Ok(Topic::Break));
    }

    #[test]
    fn test_parse_entry() {
        assert_eq!(
            "10:02".parse(),
            Ok(Entry {
                time: Time::new(10, 2).unwrap(),
                topic: Topic::Break,
            })
        );
        assert_eq!(
            "10:02 Test".parse(),
            Ok(Entry {
                time: Time::new(10, 2).unwrap(),
                topic: Topic::Project {
                    identifier: "Test".to_owned(),
                    comment: None,
                },
            })
        );
        assert_eq!(
            "10:02 Test bla bla bla".parse(),
            Ok(Entry {
                time: Time::new(10, 2).unwrap(),
                topic: Topic::Project {
                    identifier: "Test".to_owned(),
                    comment: Some("bla bla bla".to_owned()),
                },
            })
        );
        assert_eq!("10".parse::<Entry>(), Err(EntryError::Time));
        assert_eq!("".parse::<Entry>(), Err(EntryError::MissingTime));
    }
}
