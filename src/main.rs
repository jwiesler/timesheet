#![deny(rust_2018_idioms, nonstandard_style)]
#![warn(future_incompatible)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write, stdout};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::Datelike;
use clap::{Parser, Subcommand, ValueEnum};
use fs_err::File;
use thiserror::Error;
use times::convert::Month;
use times::generate::Template;
use times::parse::{from_stem, parse};

use crate::term::run_term;

mod term;

#[derive(Parser)]
struct Args {
    /// Input path timesheet
    #[clap(short, long)]
    file: Option<PathBuf>,
}

#[derive(ValueEnum, Copy, Clone)]
pub enum TemplateName {
    Empty,
    TechDay,
    Holiday,
    Normal,
    Ill,
}

impl From<TemplateName> for Template {
    fn from(value: TemplateName) -> Self {
        match value {
            TemplateName::Empty => Template::Empty,
            TemplateName::TechDay => Template::TechDay,
            TemplateName::Holiday => Template::Holiday,
            TemplateName::Normal => Template::Normal,
            TemplateName::Ill => Template::Ill,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    Check {
        #[clap(flatten)]
        args: Args,
    },
    Report {
        #[clap(flatten)]
        args: Args,
    },
    Output {
        #[clap(flatten)]
        args: Args,
    },
    Add {
        template: TemplateName,
        #[clap(flatten)]
        args: Args,
        template_args: Vec<String>,
    },
    Terminal {
        #[clap(flatten)]
        args: Args,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    commands: Option<Command>,
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to read input file: {0}")]
    InputFile(std::io::Error),
    #[error("Failed to parse input: {0}")]
    Parse(#[from] times::parse::Error),
    #[error("Invalid times: {0}")]
    Validate(#[from] times::convert::Error),
    #[error("Error running template: {0}")]
    Template(#[from] times::generate::Error),
}

fn run(cli: &Command, path: &Path) -> Result<(), Error> {
    let stem = path
        .file_stem()
        .expect("need a file with a name")
        .to_str()
        .unwrap();
    let date = from_stem(stem).unwrap_or_else(|| {
        panic!("failed to parse month from input file stem {stem:?}, expected format YYYY-MM")
    });
    let file = File::open(path).map_err(Error::InputFile)?;
    let days = parse(&mut BufReader::new(file), date)?;
    let days = days
        .into_iter()
        .map(times::convert::Day::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let month = Month::new(days);

    match cli {
        Command::Check { .. } => {}
        Command::Report { .. } => {
            let output = times::report::Output(&month);
            write!(&mut stdout(), "{output}").expect("format output");
        }
        Command::Output { .. } => {
            let output = times::format::Output(&month.days);
            write!(&mut stdout(), "{output}").expect("format output");
        }
        Command::Add {
            template,
            template_args,
            ..
        } => {
            let template: Template = (*template).into();
            let date = month
                .days
                .last()
                .and_then(|d| d.date.value.following_day_in_month())
                .unwrap_or(date)
                .next_weekday_in_month()
                .expect("last day in the month");
            let template_args = template_args.iter().map(String::as_str).collect::<Vec<_>>();
            let rendered = template.execute(date, &template_args)?;
            println!("{}", indent(&rendered));
            append_to_file(path, &rendered).map_err(Error::InputFile)?;
        }
        Command::Terminal { .. } => unreachable!(),
    }
    Ok(())
}

fn main() -> ExitCode {
    let command = Cli::parse().commands.unwrap_or(Command::Terminal {
        args: Args { file: None },
    });
    let path = match &command {
        Command::Check { args, .. }
        | Command::Report { args, .. }
        | Command::Output { args, .. }
        | Command::Terminal { args }
        | Command::Add { args, .. } => args.file.as_deref(),
    };
    let path = path.map_or_else(
        || {
            let mut cd = std::env::current_dir().unwrap();
            cd.push("timesheets");
            let now = chrono::offset::Local::now();
            let year = now.year();
            let month = now.month();
            cd.push(format!("{year}-{month:0>2}.tsh"));
            Cow::Owned(cd)
        },
        Cow::Borrowed,
    );
    let path = path.as_ref();
    if let Command::Terminal { .. } = command {
        run_term(path).unwrap();
        return ExitCode::SUCCESS;
    }
    match run(&command, path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn append_to_file(path: &Path, text: &str) -> Result<(), std::io::Error> {
    let file = OpenOptions::new().append(true).open(path)?;
    BufWriter::new(file).write_all(text.as_bytes())
}

fn indent(s: &str) -> String {
    let mut res = String::new();
    for l in s.lines() {
        res.push_str("+ ");
        res.push_str(l);
        res.push('\n');
    }
    if s.ends_with('\n') {
        res.push_str("+ ");
    }
    res
}
