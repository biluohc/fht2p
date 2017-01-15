use std::fmt::{self, Formatter};
use std::{io, fs};
use std::error;
use std::time;

extern crate  chrono;
use chrono::{Local, DateTime, Duration};

// fn main() {
//     let path = "/";
//     println!("{:?}", fms(path));
// }

// file_modified_size
pub fn fms(path: &str) -> (String, String) {
    // println!("{}", path);
    let date;
    let size;
    match fmse(path) {
        Ok(ok) => {
            let mut local: DateTime<Local> = Local::now();
            local = local - Duration::seconds(ok.0.as_secs() as i64);
            date = local.format("%Y-%m-%d %H:%M:%S").to_string();
            size = size_human(&ok.1);

        }
        Err(_) => {
            // println!("{:?}", e); 2015-06-13 12:38:05      27.13 M
            date = String::from("-- - -  ---");
            size = String::from("-- -");

        }
    };
    (date, size)
}

fn size_human(size: &u64) -> String {
    let mut tmp = size.clone();
    let mut count = 0;
    loop {
        tmp = tmp / 1024;
        if tmp == 0 {
            break;
        }
        count += 1;
    }
    let unit = match count {
        0 => "",
        1 => "K",
        2 => "M",
        3 => "G",
        4 => "T",
        5 => "P",
        _ => "+",
    };
    match count {
        0 => {
            let tmp = size;
            format!("{:.2} {}", tmp, unit)
        }
        _ => {
            let mut tmp = *size as f64;
            for _ in 0..count {
                tmp = tmp / 1024f64;
            }
            format!("{:.2} {}", tmp, unit)
        }

    }

}

// file_modified_size_Error
fn fmse(path: &str) -> Result<(time::Duration, u64), Error> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let date = modified.elapsed()?;
    let len = metadata.len();
    Ok((date, len))
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    TIME(time::SystemTimeError),
}
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(ref e) => e.description(),
            Error::TIME(ref e) => e.description(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(e: time::SystemTimeError) -> Self {
        Error::TIME(e)
    }
}
