#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub mod format;
pub mod parse;
pub mod verify;

#[derive(Debug, Eq, PartialEq)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

impl Time {
    #[must_use]
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
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
