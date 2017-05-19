extern crate time;
use time::*;

/// `Date`: `Local` and `UTC` time
#[derive(Debug,Clone)]
pub struct Time {
    local: Tm,
    utc: Tm,
}
impl Time {
    pub fn now() -> Self {
        Self::new(now())
    }
    pub fn new(tm: Tm) -> Self {
        Self {
            local: tm,
            //  utc: "Thu, 22 Mar 2012 14:53:18 GMT"
            utc: tm.to_utc(),
        }
    }
    #[inline]
    pub fn local(&self) -> &Tm {
        &self.local
    }
    /// `%H:%M:%S`
    #[inline]
    pub fn hms(&self) -> String {
        format!("{:02}::{:02}::{:02}",
                self.local.tm_hour,
                self.local.tm_min,
                self.local.tm_sec)
    }
    pub fn ls(&self) -> TmFmt {
        self.local.rfc822()
    }
    #[inline]
    pub fn utc(&self) -> &Tm {
        &self.utc
    }
    #[inline]
    pub fn us(&self) -> TmFmt {
        self.utc.rfc822()
    }
}

// stat  .gitignore
#[test]
fn test() {
    use std::fs::File;
    let f = ".gitignore";
    let std_du = File::open(f)
        .unwrap()
        .metadata()
        .unwrap()
        .modified()
        .unwrap()
        .elapsed()
        .unwrap();
    println!("{:?}", std_du);
    let time_du = Duration::from_std(std_du).unwrap();
    println!("{}", Date::new(now() - time_du).ls());
    println!("{}", Date::new(now() - time_du).us());
}
