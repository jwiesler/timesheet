#![warn(clippy::pedantic)]

use std::io::{stdout, BufReader, Write};
use std::process::ExitCode;

use times::parse::parse;

use fs_err::File;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt)]
struct Args {
    /// Input path timesheet, defaults to `timesheet.txt`
    #[structopt(default_value = "timesheet.txt")]
    path: String,
}

#[derive(StructOpt)]
enum Cli {
    Check {
        #[structopt(flatten)]
        args: Args,
    },
    Report {
        #[structopt(flatten)]
        args: Args,
    },
    Output {
        #[structopt(flatten)]
        args: Args,
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
}

fn run(cli: Cli) -> Result<(), Error> {
    let path = match &cli {
        Cli::Check { args, .. } | Cli::Report { args, .. } | Cli::Output { args, .. } => &args.path,
    };
    let file = File::open(path).map_err(Error::InputFile)?;
    let days = parse(&mut BufReader::new(file))?;
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
    }
    Ok(())
}

fn main() -> ExitCode {
    let command = Cli::from_args();
    match run(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
