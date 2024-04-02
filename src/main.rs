#![warn(clippy::pedantic)]

use clap::Parser;
use std::io::{stdout, BufReader, Write};
use std::process::ExitCode;

use times::parse::parse;

use fs_err::File;
use thiserror::Error;

#[derive(Parser)]
struct Args {
    /// Input path timesheet, defaults to `timesheet.txt`
    #[structopt(default_value = "timesheet.txt")]
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

#[allow(clippy::needless_pass_by_value)]
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
    let command = Cli::parse();
    match run(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
