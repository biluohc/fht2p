use std::time::SystemTimeError;
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::path::Path;
use std::{io, fs};

use time;
use super::Time;

#[derive(Debug)]
pub struct PathInfo {
    pub modified: Modified,
    pub len: Len,
}
pub type InfoResult<T> = Result<T, InfoError>;
#[derive(Debug)]
pub struct Modified(pub InfoResult<Time>);
#[derive(Debug)]
pub struct Len(pub InfoResult<u64>);

impl Modified {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        fn modified(path: &Path) -> InfoResult<Time> {
            let metadata = fs::metadata(path)?;
            let modified = metadata.modified()?;
            let std_duration = modified.elapsed()?;
            let duration = time::Duration::from_std(std_duration)?;
            Ok(Time::new(time::now() - duration))
        }
        Modified(modified(path.as_ref()))
    }
}

impl Display for Modified {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Ok(ref date) = self.0 {
            write!(f, "{}", date.ls())
        } else {
            f.write_str("xxx xx")
        }
    }
}

impl Display for Len {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Ok(len) = self.0 {
            write!(f, "{}", len)
        } else {
            f.write_str("xx")
        }
    }
}

impl Len {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        fn len(path: &Path) -> InfoResult<u64> {
            let metadata = fs::metadata(path)?;
            Ok(metadata.len())
        }
        Len(len(path.as_ref()))
    }
}

#[allow(unknown_lints,len_without_is_empty)]
impl PathInfo {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        PathInfo {
            modified: Modified::new(path),
            len: Len::new(path),
        }
    }
    pub fn len(&self) -> &Len {
        &self.len
    }
    pub fn modified(&self) -> &Modified {
        &self.modified
    }
}

#[derive(Debug)]
pub enum InfoError {
    IO(io::Error),
    SystemTime(SystemTimeError),
    TimeOutOfRange(time::OutOfRangeError),
}
impl Error for InfoError {
    fn description(&self) -> &str {
        match *self {
            InfoError::IO(ref e) => e.description(),
            InfoError::SystemTime(ref e) => e.description(),
            InfoError::TimeOutOfRange(ref e) => e.description(),
        }
    }
}

impl Display for InfoError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl From<io::Error> for InfoError {
    fn from(e: io::Error) -> Self {
        InfoError::IO(e)
    }
}

impl From<SystemTimeError> for InfoError {
    fn from(e: SystemTimeError) -> Self {
        InfoError::SystemTime(e)
    }
}

impl From<time::OutOfRangeError> for InfoError {
    fn from(e: time::OutOfRangeError) -> Self {
        InfoError::TimeOutOfRange(e)
    }
}
