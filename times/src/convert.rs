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

#[derive(Debug, Eq, PartialEq)]
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

impl Day {
    #[must_use]
    pub fn work_time(&self) -> Minutes {
        self.entries.iter().map(|entry| entry.value.duration).sum()
        // Reisezeit min(max(0, T[Reisezeit] – 45min) × 75%, 6h)
    }
}
