use thiserror::Error;

use crate::{Minutes, Positioned, Time, Topic};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Time span in line {0} is never terminated")]
    NotTerminated(usize),
    #[error("Minutes of time in line {0} are not a multiple of three")]
    TimeNotMultipleOfThree(usize),
    #[error("Time in line {0} ends before it starts")]
    UnorderedEntries(usize),
}

#[derive(Debug)]
#[cfg_attr(test, derive(Default, Eq, PartialEq))]
pub struct Entry {
    pub start: Positioned<Time>,
    pub end: Positioned<Time>,
    pub duration: Minutes,
    pub identifier: String,
    pub comment: Option<String>,
}

#[derive(Debug)]
pub struct Day {
    pub comments: Vec<String>,
    pub day: Positioned<String>,
    pub entries: Vec<Positioned<Entry>>,
}

impl TryFrom<crate::Day> for Day {
    type Error = Error;

    fn try_from(value: crate::Day) -> Result<Self, Self::Error> {
        let crate::Day {
            comments,
            day,
            entries,
        } = value;
        let mut new_entries = Vec::with_capacity(entries.len());
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
                let next = iter.peek().ok_or(Error::NotTerminated(entry.line))?;
                let duration = next
                    .value
                    .time
                    .elapsed(entry.value.time)
                    .ok_or(Error::UnorderedEntries(entry.line))?;
                new_entries.push(Positioned::new(
                    entry.line,
                    Entry {
                        start: Positioned::new(entry.line, entry.value.time),
                        end: Positioned::new(next.line, next.value.time),
                        duration,
                        identifier,
                        comment,
                    },
                ));
            }
        }
        Ok(Day {
            comments,
            day,
            entries: new_entries,
        })
    }
}

#[derive(Default)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct AccumulatedTime {
    travel: Minutes,
    work: Minutes,
}

fn billable_travel_time(travel: Minutes) -> Minutes {
    let minutes = travel.into_inner();
    if let Some(m) = minutes.checked_sub(45) {
        Minutes::from(((m * 3) / 4).min(6 * 60))
    } else {
        Minutes::default()
    }
}

impl AccumulatedTime {
    #[must_use]
    pub fn billable_travel_time(&self) -> Minutes {
        billable_travel_time(self.travel)
    }

    #[must_use]
    pub fn work_time(&self) -> Minutes {
        self.work
    }
}

impl Day {
    #[must_use]
    pub fn accumulated_time(&self) -> AccumulatedTime {
        self.entries
            .iter()
            .fold(AccumulatedTime::default(), |acc, entry| {
                let AccumulatedTime { travel, work } = acc;
                let duration = entry.value.duration;
                if &entry.value.identifier == "TNGFa" {
                    AccumulatedTime {
                        travel: travel + duration,
                        work,
                    }
                } else {
                    AccumulatedTime {
                        work: work + duration,
                        travel,
                    }
                }
            })
        // Reisezeit min(max(0, T[Reisezeit] – 45min) × 75%, 6h)
    }
}

#[cfg(test)]
mod test {
    use crate::convert::{billable_travel_time, AccumulatedTime, Day, Entry};
    use crate::{Minutes, Positioned};

    #[test]
    fn test_travel_time_calc() {
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

    #[test]
    fn test_accumulated_time() {
        let day = Day {
            comments: vec![],
            day: Positioned::new(0, "A".into()),
            entries: vec![
                Positioned::new(
                    0,
                    Entry {
                        duration: 60.into(),
                        identifier: "TNG".to_string(),
                        ..Default::default()
                    },
                ),
                Positioned::new(
                    0,
                    Entry {
                        duration: 30.into(),
                        identifier: "TNGFa".to_string(),
                        ..Default::default()
                    },
                ),
                Positioned::new(
                    0,
                    Entry {
                        duration: 60.into(),
                        identifier: "TNG".to_string(),
                        ..Default::default()
                    },
                ),
                Positioned::new(
                    0,
                    Entry {
                        duration: 30.into(),
                        identifier: "TNGFa".to_string(),
                        ..Default::default()
                    },
                ),
            ],
        };

        assert_eq!(
            day.accumulated_time(),
            AccumulatedTime {
                travel: 60.into(),
                work: 120.into()
            }
        )
    }
}
