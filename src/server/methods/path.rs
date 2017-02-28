use std::time::{Duration, SystemTimeError};
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::{io, fs};

type PathResult<T> = Result<T, PathError>;

pub fn pathsm(path: &str) -> PathResult<(u64, Duration)> {
    let metadata = fs::metadata(path)?;
    let len = metadata.len();
    let modified = metadata.modified()?;
    let std_duration = modified.elapsed()?;
    Ok((len, std_duration))
}

#[derive(Debug)]
pub enum PathError {
    IO(io::Error),
    TIME(SystemTimeError),
}
impl Error for PathError {
    fn description(&self) -> &str {
        match *self {
            PathError::IO(ref e) => e.description(),
            PathError::TIME(ref e) => e.description(),
        }
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl From<io::Error> for PathError {
    fn from(e: io::Error) -> Self {
        PathError::IO(e)
    }
}

impl From<SystemTimeError> for PathError {
    fn from(e: SystemTimeError) -> Self {
        PathError::TIME(e)
    }
}

// #[test]
fn main() {
    let p = "/2016-0608";
    println!("{:?}", pathsm(p));
}
