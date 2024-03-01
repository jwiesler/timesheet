#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod format;
pub mod parse;
pub mod verify;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
    pub fn elapsed_minutes(self, o: Time) -> Option<usize> {
        let hours = self.hour.checked_sub(o.hour)?;
        let hour_minutes = usize::from(hours) * 60;
        let minutes = if self.minute >= o.minute {
            hour_minutes + usize::from(self.minute.checked_sub(o.minute).unwrap())
        } else {
            hour_minutes.checked_sub(usize::from(o.minute.checked_sub(self.minute).unwrap()))?
        };
        Some(minutes)
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

#[derive(Debug, Clone, Eq, PartialEq)]
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
    fn test_elapsed_minutes() {
        assert_eq!(
            Time::new(12, 00)
                .unwrap()
                .elapsed_minutes(Time::new(11, 00).unwrap()),
            Some(60)
        );
        assert_eq!(
            Time::new(12, 10)
                .unwrap()
                .elapsed_minutes(Time::new(11, 5).unwrap()),
            Some(65)
        );
        assert_eq!(
            Time::new(12, 5)
                .unwrap()
                .elapsed_minutes(Time::new(11, 10).unwrap()),
            Some(55)
        );
        assert_eq!(
            Time::new(11, 00)
                .unwrap()
                .elapsed_minutes(Time::new(12, 00).unwrap()),
            None
        );
        assert_eq!(
            Time::new(12, 00)
                .unwrap()
                .elapsed_minutes(Time::new(12, 01).unwrap()),
            None
        );
    }
}
