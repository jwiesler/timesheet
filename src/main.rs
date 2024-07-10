#![warn(clippy::pedantic)]

use std::fs::OpenOptions;
use std::io::{stdout, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use fs_err::File;
use thiserror::Error;

use times::generate::Template;
use times::parse::{filename, parse};

#[derive(Parser)]
struct Args {
    /// Input path timesheet
    path: String,
}

#[derive(Parser)]
enum Cli {
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
        template: String,
        #[clap(flatten)]
        args: Args,
        template_args: Vec<String>,
    },
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

fn run(cli: &Cli) -> Result<(), Error> {
    let path = match cli {
        Cli::Check { args, .. }
        | Cli::Report { args, .. }
        | Cli::Output { args, .. }
        | Cli::Add { args, .. } => &args.path,
    };
    let path = Path::new(path);
    let stem = path
        .file_stem()
        .expect("need a file with a name")
        .to_str()
        .unwrap();
    let month = filename(stem).unwrap_or_else(|| {
        panic!("failed to parse month from input file stem {stem:?}, expected format YYYY-MM")
    });
    let file = File::open(path).map_err(Error::InputFile)?;
    let days = parse(&mut BufReader::new(file), month)?;
    let days = days
        .into_iter()
        .map(times::convert::Day::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    match cli {
        Cli::Check { .. } => {}
        Cli::Report { .. } => {
            let output = times::report::Output(&days);
            write!(&mut stdout(), "{output}").expect("format output");
        }
        Cli::Output { .. } => {
            let output = times::format::Output(&days);
            write!(&mut stdout(), "{output}").expect("format output");
        }
        Cli::Add {
            template,
            template_args,
            ..
        } => {
            let template = Template::by_name(template)?;
            let date = days
                .last()
                .and_then(|d| d.day.value.following_day_in_month())
                .unwrap_or(month)
                .next_weekday_in_month()
                .expect("last day in the month");
            let rendered = template.execute(date, template_args)?;
            println!("{}", indent(&rendered));
            append_to_file(path, &rendered).map_err(Error::InputFile)?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let command = Cli::parse();
    match run(&command) {
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
