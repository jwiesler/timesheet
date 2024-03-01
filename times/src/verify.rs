use thiserror::Error;

use crate::{Day, Topic};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Time span in line {0} is never terminated")]
    NotTerminated(usize),
    #[error("Minutes of time in line {0} are not a multiple of three")]
    TimeNotMultipleOfThree(usize),
}

pub fn verify(days: &[Day]) -> Result<(), Error> {
    for day in days {
        if let Some(last) = day.entries.last() {
            if !matches!(last.value.topic, Topic::Break) {
                return Err(Error::NotTerminated(last.line));
            }
        }
        day.entries.iter().try_for_each(|e| {
            if e.value.time.minute % 3 != 0 {
                Err(Error::TimeNotMultipleOfThree(e.line))
            } else {
                Ok(())
            }
        })?;
    }
    Ok(())
}
