use std::fmt::{Display, Formatter, Result};

use anstyle::{AnsiColor, Color, Style};

use crate::convert::{Day, Entry, Month};
use crate::{Minutes, Positioned};

const DATE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)));
const PROJECT: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)));
const TIME: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta)));
const POSITIVE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const NEGATIVE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
const ADDITIONS: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));

pub struct Output<'a>(pub &'a Month);

pub trait Format {
    fn format(&self, f: &mut Formatter<'_>) -> Result;
}

impl Display for Output<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.format(f)
    }
}

fn output_time_delta(f: &mut Formatter<'_>, lhs: Minutes, rhs: Minutes) -> Result {
    if lhs < rhs {
        let delta = rhs - lhs;
        let duration = delta.into_duration();
        write!(
            f,
            "{}-{duration}{}",
            NEGATIVE.render(),
            NEGATIVE.render_reset()
        )
    } else {
        let delta = lhs - rhs;
        let duration = delta.into_duration();
        write!(
            f,
            "{}+{duration}{}",
            POSITIVE.render(),
            POSITIVE.render_reset()
        )
    }
}

impl Format for &'_ Month {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        self.days.as_slice().format(f)?;
        let minutes = self.times.billable_time();
        let duration = minutes.into_duration();
        writeln!(f)?;
        write!(f, "{}Total: {duration} (", ADDITIONS.render())?;
        output_time_delta(f, minutes, self.expected_min_work)?;
        writeln!(f, "{}){}", ADDITIONS.render(), ADDITIONS.render_reset())?;
        Ok(())
    }
}

impl Format for &'_ [Day] {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        let mut first = true;
        for day in *self {
            if day.entries.is_empty() {
                continue;
            }
            if first {
                first = false;
            } else {
                writeln!(f)?;
            }
            day.format(f)?;
        }
        Ok(())
    }
}

impl Format for [Positioned<Entry>] {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        for entry in self {
            entry.value.format(f)?;
        }
        Ok(())
    }
}

impl Format for Entry {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}{} - {}{} {}{}{}",
            TIME.render(),
            self.start.value,
            self.end.value,
            TIME.render_reset(),
            PROJECT.render(),
            self.identifier,
            PROJECT.render_reset(),
        )?;
        if let Some(comment) = &self.comment {
            write!(f, " {comment}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Format for &'_ Day {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}* {}{}",
            DATE.render(),
            self.date.value,
            DATE.render_reset()
        )?;
        let minutes = self.times.billable_time();
        if minutes == Minutes::default() {
            writeln!(f)?;
        } else {
            let duration = minutes.into_duration();
            write!(f, "{} -> {duration}", ADDITIONS.render())?;
            let expected_time = self.expected_time();
            if minutes == expected_time {
                writeln!(f, "{}", ADDITIONS.render_reset())?;
            } else {
                write!(f, " (")?;
                output_time_delta(f, minutes, expected_time)?;
                writeln!(f, "{}){}", ADDITIONS.render(), ADDITIONS.render_reset())?;
            }
        }

        self.entries.as_slice().format(f)?;

        if self.times.travel_time() > Minutes::default() {
            writeln!(
                f,
                "{}Travel time: {} ({} billable){}",
                ADDITIONS.render(),
                self.times.travel_time().into_duration(),
                self.times.billable_travel_time().into_duration(),
                ADDITIONS.render_reset(),
            )?;
        }

        Ok(())
    }
}
