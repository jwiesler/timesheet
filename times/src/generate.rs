use std::fmt::{Arguments, Write};

use thiserror::Error;

use crate::Date;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("Unknown template")]
    UnknownTemplate,
    #[error("Argument count mismatch, expected {0}, got {1}")]
    Argc(usize, usize),
}

pub enum Template {
    Empty,
    TechDay,
    Holiday,
    Normal,
}

struct Builder(String);

impl Builder {
    fn new() -> Self {
        Self(String::new())
    }

    fn header(&mut self, date: Date) -> &mut Self {
        self.line_fmt(format_args!("\n* {date}"))
    }

    fn line_fmt(&mut self, args: Arguments<'_>) -> &mut Self {
        self.0.write_fmt(args).unwrap();
        self.0.push('\n');
        self
    }

    fn line(&mut self, line: &str) -> &mut Self {
        self.0.push_str(line);
        self.0.push('\n');
        self
    }
}

impl Template {
    pub fn by_name(name: &str) -> Result<Self, Error> {
        match name {
            "empty" => Ok(Template::Empty),
            "techday" => Ok(Template::TechDay),
            "holiday" => Ok(Template::Holiday),
            "normal" => Ok(Template::Normal),
            _ => Err(Error::UnknownTemplate),
        }
    }

    pub fn execute(&self, date: Date, args: &[String]) -> Result<String, Error> {
        let mut output = Builder::new();
        match self {
            Template::Empty => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                output.header(date);
            }
            Template::TechDay => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                output
                    .header(date)
                    .line("09:00 TNGFo Techday")
                    .line("17:00");
            }
            Template::Holiday => {
                if !args.is_empty() {
                    return Err(Error::Argc(0, args.len()));
                }
                output.header(date).line("09:00 Urlaub").line("17:00");
            }
            Template::Normal => {
                if args.is_empty() || 2 < args.len() {
                    return Err(Error::Argc(2, args.len()));
                }
                let arg_0 = &args[0];
                let arg_1 = args.get(1).unwrap_or(arg_0);

                output
                    .header(date)
                    .line("09:00 AA Ops Daily")
                    .line("09:15 AA Inference Daily")
                    .line_fmt(format_args!("09:45 AA {arg_0}"))
                    .line("12:30")
                    .line_fmt(format_args!("13:00 AA {arg_1}"))
                    .line("17:30");
            }
        }

        Ok(output.0)
    }
}
