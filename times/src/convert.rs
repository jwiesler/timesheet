use std::fmt::{Display, Formatter};
use std::ops::Add;

use thiserror::Error;

use crate::{Date, Minutes, Positioned, String, Time, Topic};

#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("Time span in line {0} is never terminated")]
    NotTerminated(usize),
    #[error("Minutes of time in line {0} are not a multiple of three")]
    TimeNotMultipleOfThree(usize),
    #[error("Time in line {0} ends before it starts")]
    EndsBeforeItStarts(usize),
    #[error("Time in line {0} overlaps with the time before it")]
    OverlapWithPrevious(usize),
    #[error("Time in line {0} crosses the start of end of a previous travel time")]
    AcrossTravelTime(usize),
}

#[cfg_attr(test, derive(Default, Clone, Eq, PartialEq))]
pub struct Identifier(String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Identifier {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn is_tng(&self) -> bool {
        self.0.starts_with("TNG")
    }

    #[must_use]
    pub fn is_travel(&self) -> bool {
        self.0.ends_with("Fa")
    }

    #[must_use]
    pub fn is_under_hours(&self) -> bool {
        self.0.starts_with("Ustd")
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg_attr(test, derive(Default, Eq, PartialEq))]
pub struct Entry {
    pub start: Positioned<Time>,
    pub end: Positioned<Time>,
    pub duration: Minutes,
    pub identifier: Identifier,
    pub comment: Option<String>,
}

pub struct Day {
    pub comments: Vec<String>,
    pub date: Positioned<Date>,
    pub entries: Vec<Positioned<Entry>>,
    pub times: AccumulatedTime,
}

impl Day {
    #[must_use]
    pub fn expected_time(&self) -> Minutes {
        if self.date.value.is_weekday() && !self.entries.is_empty() {
            Minutes::from_hours(8)
        } else {
            Minutes::default()
        }
    }
}

pub struct Month {
    pub days: Vec<Day>,
    pub expected_min_work: Minutes,
    pub times: AccumulatedTime,
}

impl Month {
    pub fn new(days: Vec<Day>) -> Self {
        let expected_min_work = days
            .iter()
            .filter(|d| !d.entries.is_empty())
            .map(Day::expected_time)
            .sum();

        let time = days
            .iter()
            .map(|d| d.times.clone())
            .fold(AccumulatedTime::default(), AccumulatedTime::add);
        Self {
            days,
            expected_min_work,
            times: time,
        }
    }
}

#[must_use]
fn accumulated_time<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> AccumulatedTime {
    let mut last_travel = None;
    entries
        .into_iter()
        .fold(AccumulatedTime::default(), |acc, entry| {
            let AccumulatedTime { travel, work } = acc;
            let duration = entry.duration;
            if entry.identifier.is_travel() {
                last_travel = Some(entry);
            }
            if entry.identifier.is_under_hours() {
                AccumulatedTime { travel, work }
            } else if entry.identifier.is_travel() {
                let travel = if entry.identifier.is_tng() {
                    TravelTime {
                        tng: travel.tng + duration,
                        other: travel.other,
                    }
                } else {
                    TravelTime {
                        tng: travel.tng,
                        other: travel.other + duration,
                    }
                };
                AccumulatedTime { travel, work }
            } else if let Some(last_travel) = last_travel
                .filter(|t| t.start.value <= entry.start.value && entry.end.value <= t.end.value)
            {
                let travel = if last_travel.identifier.is_tng() {
                    TravelTime {
                        tng: travel.tng - duration,
                        other: travel.other,
                    }
                } else {
                    TravelTime {
                        tng: travel.tng,
                        other: travel.other - duration,
                    }
                };

                AccumulatedTime {
                    work: work + duration,
                    travel,
                }
            } else {
                AccumulatedTime {
                    work: work + duration,
                    travel,
                }
            }
        })
}

fn validate_ordering(
    entry: &Entry,
    previous: &Entry,
    last_travel: Option<&Entry>,
) -> Result<(), Error> {
    if let Some(last_travel) = last_travel {
        if entry.start.value < last_travel.start.value
            || (entry.start.value < last_travel.end.value
                && last_travel.end.value < entry.end.value)
        {
            return Err(Error::AcrossTravelTime(entry.start.line));
        }
        if std::ptr::eq(last_travel, previous) {
            return Ok(());
        }
    }
    if previous.end.value <= entry.start.value {
        Ok(())
    } else {
        Err(Error::OverlapWithPrevious(entry.start.line))
    }
}

impl TryFrom<crate::Day> for Day {
    type Error = Error;

    fn try_from(value: crate::Day) -> Result<Self, Self::Error> {
        let crate::Day {
            comments,
            date,
            entries,
        } = value;
        let mut new_entries: Vec<Positioned<Entry>> = Vec::with_capacity(entries.len());
        let mut last_travel = None;
        let mut iter = entries.into_iter().peekable();
        while let Some(entry) = iter.next() {
            if entry.value.time.minute % 3 != 0 {
                return Err(Error::TimeNotMultipleOfThree(entry.line));
            }
            if let Topic::Project {
                identifier,
                comment,
            } = entry.value.topic
            {
                let identifier = Identifier(identifier);
                let next = iter.peek().ok_or(Error::NotTerminated(entry.line))?;
                let duration = next
                    .value
                    .time
                    .elapsed(entry.value.time)
                    .ok_or(Error::EndsBeforeItStarts(entry.line))?;
                let new_entry = Entry {
                    start: Positioned::new(entry.line, entry.value.time),
                    end: Positioned::new(next.line, next.value.time),
                    duration,
                    identifier,
                    comment,
                };

                if new_entry.identifier.is_travel() {
                    last_travel = Some(new_entries.len());
                }
                if let Some(previous_entry) = new_entries.last() {
                    validate_ordering(
                        &new_entry,
                        &previous_entry.value,
                        last_travel
                            .and_then(|i: usize| new_entries.get(i))
                            .map(|e| &e.value),
                    )?;
                }

                new_entries.push(Positioned::new(entry.line, new_entry));
            }
        }

        let times = accumulated_time(new_entries.iter().map(|e| &e.value));
        Ok(Day {
            comments,
            date,
            entries: new_entries,
            times,
        })
    }
}

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
struct TravelTime {
    tng: Minutes,
    other: Minutes,
}

impl Add for TravelTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        TravelTime {
            tng: self.tng + rhs.tng,
            other: self.other + rhs.other,
        }
    }
}

impl TravelTime {
    fn billable(&self) -> Minutes {
        let tng = billable_travel_time(self.tng);
        tng + self.other
    }

    fn total(&self) -> Minutes {
        self.tng + self.other
    }
}

fn billable_travel_time(minutes: Minutes) -> Minutes {
    let minutes = minutes.into_inner();
    if let Some(m) = minutes.checked_sub(45) {
        Minutes::from(((m * 3) / 4).min(6 * 60))
    } else {
        Minutes::default()
    }
}

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct AccumulatedTime {
    travel: TravelTime,
    work: Minutes,
}

impl AccumulatedTime {
    #[must_use]
    pub fn billable_travel_time(&self) -> Minutes {
        self.travel.billable()
    }

    #[must_use]
    pub fn travel_time(&self) -> Minutes {
        self.travel.total()
    }

    #[must_use]
    pub fn work_time(&self) -> Minutes {
        self.work
    }

    #[must_use]
    pub fn billable_time(&self) -> Minutes {
        self.work + self.billable_travel_time()
    }
}

impl Add<AccumulatedTime> for AccumulatedTime {
    type Output = Self;

    fn add(self, rhs: AccumulatedTime) -> Self::Output {
        AccumulatedTime {
            travel: self.travel + rhs.travel,
            work: self.work + rhs.work,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::convert::{
        accumulated_time, billable_travel_time, validate_ordering, AccumulatedTime, Entry, Error,
        Identifier, TravelTime,
    };
    use crate::{Minutes, Positioned, Time};

    #[test]
    fn travel_time_calc() {
        assert_eq!(billable_travel_time(Minutes::from(20)), Minutes::default());
        assert_eq!(
            billable_travel_time(Minutes::from(4 * 60 + 45)),
            Minutes::from(3 * 60)
        );
        assert_eq!(
            billable_travel_time(Minutes::from(9 * 60)),
            Minutes::from(6 * 60)
        );
    }

    fn new_entry(start: Option<Time>, end: Option<Time>, identifier: &str) -> Entry {
        Entry {
            start: Positioned::new(0, start.unwrap()),
            end: Positioned::new(0, end.unwrap()),
            duration: end.unwrap().elapsed(start.unwrap()).unwrap(),
            identifier: Identifier(identifier.into()),
            comment: None,
        }
    }

    #[test]
    fn accumulated_travel_time() {
        let entries = [
            new_entry(Time::new(0, 0), Time::new(1, 0), "TNG"),
            new_entry(Time::new(1, 0), Time::new(1, 30), "TNGFa"),
            new_entry(Time::new(1, 30), Time::new(2, 30), "TNG"),
        ];

        assert_eq!(
            accumulated_time(&entries),
            AccumulatedTime {
                travel: TravelTime {
                    tng: 30.into(),
                    other: 0.into()
                },
                work: 120.into(),
            }
        );
    }

    #[test]
    fn test_accumulated_under_hours() {
        let entries = [
            new_entry(Time::new(1, 30), Time::new(2, 0), "TNG"),
            new_entry(Time::new(2, 0), Time::new(2, 30), "TNGFa"),
            new_entry(Time::new(2, 30), Time::new(3, 0), "Ustd"),
            new_entry(Time::new(3, 0), Time::new(3, 30), "UstdPart"),
            new_entry(Time::new(3, 30), Time::new(4, 0), "TNGFa"),
        ];

        assert_eq!(
            accumulated_time(&entries),
            AccumulatedTime {
                travel: TravelTime {
                    tng: 60.into(),
                    other: 0.into()
                },
                work: 30.into(),
            }
        );
    }

    #[test]
    fn accumulated_overlapping_travel_times() {
        let entries = [
            new_entry(Time::new(3, 0), Time::new(4, 0), "TNG"),
            new_entry(Time::new(4, 0), Time::new(4, 30), "TNGFa"),
            new_entry(Time::new(4, 0), Time::new(4, 10), "TNG"),
            new_entry(Time::new(4, 15), Time::new(4, 20), "TNG"),
            new_entry(Time::new(4, 20), Time::new(4, 30), "TNG"),
            new_entry(Time::new(5, 0), Time::new(5, 30), "AAFa"),
            new_entry(Time::new(5, 0), Time::new(5, 15), "AA"),
        ];

        assert_eq!(
            accumulated_time(&entries),
            AccumulatedTime {
                travel: TravelTime {
                    tng: 5.into(),
                    other: 15.into(),
                },
                work: 100.into(),
            }
        );
    }

    #[test]
    fn ordering() {
        let previous_entry = new_entry(Time::new(1, 0), Time::new(2, 0), "TNG");
        let entry = new_entry(Time::new(2, 0), Time::new(2, 30), "TNG");
        assert_eq!(validate_ordering(&entry, &previous_entry, None), Ok(()));

        let entry = new_entry(Time::new(1, 59), Time::new(2, 30), "TNG");
        assert_eq!(
            validate_ordering(&entry, &previous_entry, None),
            Err(Error::OverlapWithPrevious(0))
        );
    }

    #[test]
    fn ordering_travel() {
        let travel = new_entry(Time::new(1, 0), Time::new(2, 30), "TNGFa");

        // After travel
        let entry = new_entry(Time::new(2, 30), Time::new(3, 00), "TNG");
        assert_eq!(validate_ordering(&entry, &travel, Some(&travel)), Ok(()));

        // Overlaps and fully inside
        let entry = new_entry(Time::new(1, 30), Time::new(2, 00), "TNG");
        assert_eq!(validate_ordering(&entry, &travel, Some(&travel)), Ok(()));

        // Overlaps and starts before travel
        let entry = new_entry(Time::new(0, 30), Time::new(1, 30), "TNG");
        assert_eq!(
            validate_ordering(&entry, &travel, Some(&travel)),
            Err(Error::AcrossTravelTime(0))
        );

        // Overlaps and ends after travel
        let entry = new_entry(Time::new(2, 0), Time::new(3, 0), "TNG");
        assert_eq!(
            validate_ordering(&entry, &travel, Some(&travel)),
            Err(Error::AcrossTravelTime(0))
        );

        let previous_entry = new_entry(Time::new(1, 30), Time::new(2, 0), "TNG");

        // Inside travel and after previous
        let entry = new_entry(Time::new(2, 00), Time::new(2, 30), "TNG");
        assert_eq!(
            validate_ordering(&entry, &previous_entry, Some(&travel)),
            Ok(())
        );

        // Inside travel and overlaps previous
        let entry = new_entry(Time::new(1, 45), Time::new(2, 30), "TNG");
        assert_eq!(
            validate_ordering(&entry, &previous_entry, Some(&travel)),
            Err(Error::OverlapWithPrevious(0))
        );
    }
}
