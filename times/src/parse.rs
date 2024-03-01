use crate::{Day, Entry, Positioned, Time, Topic};

use std::io::BufRead;
use std::mem::take;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum EntryError {
    #[error("Invalid time format")]
    Time,
    #[error("Missing time")]
    MissingTime,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse entry in line {0}: {1}")]
    Entry(usize, EntryError),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Expected a day in line {0}")]
    ExpectedDay(usize),
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

pub fn parse(r: impl BufRead) -> Result<Vec<Day>, Error> {
    let mut days = Vec::new();
    let mut current_day = None;
    let mut comments = Vec::new();
    for (index, line) in r.lines().enumerate() {
        let index = index + 1;
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(line) = line.strip_prefix('*') {
            if let Some(day) = current_day.take() {
                days.push(day);
            }
            current_day = Some(Day {
                comments: take(&mut comments),
                day: Positioned::new(index, line.trim_start().to_owned()),
                entries: Vec::new(),
            });
            continue;
        }
        let day = current_day
            .as_mut()
            .ok_or_else(|| Error::ExpectedDay(index))?;
        let entry = line.parse().map_err(|e| Error::Entry(index, e))?;
        day.entries.push(Positioned::new(index, entry));
    }
    if let Some(day) = current_day.take() {
        days.push(day);
    }
    Ok(days)
}

#[cfg(test)]
mod test {
    use crate::parse::{EntryError, TimeError};
    use crate::{Entry, Time, Topic};

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
