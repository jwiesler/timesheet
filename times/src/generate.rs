use std::fmt::{Display, Formatter, Write};

use thiserror::Error;

use crate::Date;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("Unknown template")]
    UnknownTemplate,
    #[error("Argument count mismatch, expected {0}, got {1}")]
    Argc(usize, usize),
}

#[derive(Debug)]
pub enum Template {
    Empty,
    TechDay,
    Holiday,
    Normal,
    Ill,
    TNGWeekly,
}

trait FormatterEx {
    fn header(&mut self, date: Date) -> std::fmt::Result;
}

impl FormatterEx for Formatter<'_> {
    fn header(&mut self, date: Date) -> std::fmt::Result {
        writeln!(self, "\n* {date}")
    }
}

impl Template {
    fn full_day(output: &mut String, date: Date, what: &str) -> std::fmt::Result {
        write_with(output, |f| {
            f.header(date)?;
            writeln!(f, "09:00 {what}")?;
            writeln!(f, "17:00")
        })
    }

    pub fn execute(&self, date: Date, args: &[String]) -> Result<String, Error> {
        let mut output = String::new();

        match self {
            Template::Empty => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                write_with(&mut output, |f| f.header(date)).unwrap();
            }
            Template::TechDay => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                Self::full_day(&mut output, date, "TNGFo Techday").unwrap();
            }
            Template::Holiday => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                Self::full_day(&mut output, date, "Urlaub").unwrap();
            }
            Template::Ill => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                Self::full_day(&mut output, date, "Krank").unwrap();
            }
            Template::Normal => {
                if args.is_empty() || 2 < args.len() {
                    return Err(Error::Argc(2, args.len()));
                }
                let arg_0 = &args[0];
                let arg_1 = args.get(1).unwrap_or(arg_0);

                write_with(&mut output, |f| {
                    f.header(date)?;
                    writeln!(f, "09:00 AA {arg_0}")?;
                    writeln!(f, "10:00 AA Ops Daily")?;
                    writeln!(f, "10:15 AA Inference Daily")?;
                    writeln!(f, "10:30 AA {arg_0}")?;
                    writeln!(f, "12:30")?;
                    writeln!(f, "13:00 AA {arg_1}")?;
                    writeln!(f, "17:30")
                })
                .unwrap();
            }
            Template::TNGWeekly => {
                if args.is_empty() || 2 < args.len() {
                    return Err(Error::Argc(2, args.len()));
                }
                let arg_0 = &args[0];
                let arg_1 = args.get(1).unwrap_or(arg_0);

                write_with(&mut output, |f| {
                    f.header(date)?;
                    writeln!(f, "09:00 AA {arg_0}")?;
                    writeln!(f, "10:00 AA Ops Daily")?;
                    writeln!(f, "10:15 AA Inference Daily")?;
                    writeln!(f, "10:30 AANB TNG Weekly")?;
                    writeln!(f, "10:36 AA TNG Weekly")?;
                    writeln!(f, "11:00 AA {arg_0}")?;
                    writeln!(f, "12:30")?;
                    writeln!(f, "13:00 AA {arg_1}")?;
                    writeln!(f, "17:30")
                })
                .unwrap();
            }
        }

        Ok(output)
    }
}

struct WithFormatter<F>(F);

impl<F: for<'a> Fn(&mut Formatter<'a>) -> std::fmt::Result> Display for WithFormatter<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0(f)
    }
}

fn write_with<F: for<'a> Fn(&mut Formatter<'a>) -> std::fmt::Result>(
    r: &mut String,
    f: F,
) -> std::fmt::Result {
    write!(r, "{}", WithFormatter(f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn generate() {
        let date = Date::new(NaiveDate::from_ymd_opt(2024, 8, 5).unwrap());
        let tests = [
            (Template::Empty, vec![], "\n* Mo. 5.08.\n"),
            (Template::TechDay, vec![], "\n* Mo. 5.08.\n09:00 TNGFo Techday\n17:00\n"),
            (Template::Holiday, vec![], "\n* Mo. 5.08.\n09:00 Urlaub\n17:00\n"),
            (Template::TNGWeekly, vec!["A".into()], "\n* Mo. 5.08.\n09:00 AA A\n10:00 AA Ops Daily\n10:15 AA Inference Daily\n10:30 AANB TNG Weekly\n10:36 AA TNG Weekly\n11:00 AA A\n12:30\n13:00 AA A\n17:30\n"),
            (Template::Normal, vec!["A".into()], "\n* Mo. 5.08.\n09:00 AA A\n10:00 AA Ops Daily\n10:15 AA Inference Daily\n10:30 AA A\n12:30\n13:00 AA A\n17:30\n"),
            (Template::Normal, vec!["A".into(), "B".into()], "\n* Mo. 5.08.\n09:00 AA A\n10:00 AA Ops Daily\n10:15 AA Inference Daily\n10:30 AA A\n12:30\n13:00 AA B\n17:30\n"),
            (Template::Ill, vec![], "\n* Mo. 5.08.\n09:00 Krank\n17:00\n")
        ];
        for (template, args, result) in tests {
            assert_eq!(
                template.execute(date, &args).as_deref(),
                Ok(result),
                "{template:?}{args:?}"
            );
        }
    }
}
