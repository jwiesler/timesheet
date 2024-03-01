#![warn(clippy::pedantic)]

use std::io::{BufReader, BufWriter, Write};
use std::process::ExitCode;

use times::format::Output;
use times::parse::parse;

use fs_err::{File, OpenOptions};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, StructOpt)]
struct Command {
    /// Input path timesheet, defaults to `timesheet.txt`
    #[structopt(default_value = "timesheet.txt")]
    path: String,

    /// Input path timesheet, defaults to `timesheet.txt`
    #[structopt(default_value = "timesheet.out.txt")]
    out: String,

    /// Whether to only check the input
    #[structopt(long)]
    check: bool,
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to read input file: {0}")]
    InputFile(std::io::Error),
    #[error("Failed to write to output file: {0}")]
    OutputFile(std::io::Error),
    #[error("Failed to parse input: {0}")]
    Parse(#[from] times::parse::Error),
    #[error("Invalid times: {0}")]
    Validate(#[from] times::verify::Error),
}

fn run(Command { path, out, check }: Command) -> Result<(), Error> {
    let file = File::open(path).map_err(Error::InputFile)?;
    let days = parse(&mut BufReader::new(file))?;
    let output = Output::new(&days)?;

    if !check {
        let out = OpenOptions::new()
            .write(true)
            .create(true)
            .open(out)
            .map_err(Error::OutputFile)?;
        write!(&mut BufWriter::new(out), "{output}").expect("format output");
    }
    Ok(())
}

fn main() -> ExitCode {
    let command = Command::from_args();
    match run(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
