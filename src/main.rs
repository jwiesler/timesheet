#![warn(clippy::pedantic)]

use std::io::{BufReader, BufWriter, Write};
use std::process::ExitCode;

use times::format::Output;

use fs_err::{File, OpenOptions};
use structopt::StructOpt;
use times::parse::parse;

#[derive(Debug, StructOpt)]
struct Command {
    /// Input path timesheet, defaults to `timesheet.txt`
    #[structopt(default_value = "timesheet.txt")]
    path: String,

    /// Input path timesheet, defaults to `timesheet.txt`
    #[structopt(default_value = "timesheet.out.txt")]
    out: String,
}

fn main() -> ExitCode {
    let Command { path, out } = Command::from_args();

    let file = File::open(path).expect("Failed to read input file");
    let days = match parse(&mut BufReader::new(file)) {
        Ok(days) => days,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let out = OpenOptions::new()
        .write(true)
        .create(true)
        .open(out)
        .expect("open output file");
    write!(&mut BufWriter::new(out), "{}", Output(&days)).unwrap();
    ExitCode::SUCCESS
}
