use std::fmt::{Display, Formatter, Result};

use crate::convert::{Day, Entry};
use crate::{Positioned, Time};

pub struct Output<'a>(pub &'a [Day]);

pub trait Format {
    fn format(&self, f: &mut Formatter<'_>) -> Result;
}

impl Display for Output<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.format(f)
    }
}

impl Format for &'_ [Day] {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        let mut first = true;
        for day in *self {
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

impl Format for &Day {
    fn format(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "* {}", self.date.value)?;
        self.entries.as_slice().format(f)?;

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
            "{} - {} {}",
            self.start.value, self.end.value, self.identifier
        )?;
        if let Some(comment) = &self.comment {
            write!(f, " {comment}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:0>2}:{:0>2}", self.hour, self.minute)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use chrono::NaiveDate;

    use super::*;
    use crate::parse::parse;
    use crate::Date;

    #[test]
    fn test_format() {
        let text = r"
        * Sa. 20.04.
        09:00 AA A
        12:30
        13:00 AANB B
        15:00 TNG C
        17:30
        ";
        let days = parse(
            &mut BufReader::new(Cursor::new(text)),
            Date(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
        )
        .unwrap();
        let days = days
            .into_iter()
            .map(Day::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();

        let expected = r"* Sa. 20.04.
09:00 - 12:30 AA A
13:00 - 15:00 AANB B
15:00 - 17:30 TNG C
";
        assert_eq!(format!("{}", Output(&days)), expected);
    }
}
