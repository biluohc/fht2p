extern crate time;
use time::*;

#[derive(Debug,Clone)]
pub struct Date {
    local: Tm,
    ls: String,
    utc: Tm,
    us: String,
}
impl Date {
    pub fn now() -> Self {
        Self::new(now())
    }
    pub fn new(tm: Tm) -> Self {
        let local = tm.to_local();
        Self {
            local: local,
            // "%Y-%m%d %H:%M:%S" <=> 2017-0225 00:22:30
            ls: format!("{:04}-{:02}{:02} {:02}:{:02}:{:02}",
                        local.tm_year + 1900,
                        local.tm_mon + 1,
                        local.tm_mday,
                        local.tm_hour,
                        local.tm_min,
                        local.tm_sec),
            //  utc: "Thu, 22 Mar 2012 14:53:18 GMT"
            utc: tm.to_utc(),
            us: format!("{}", tm.to_utc().rfc822()),
        }
    }
    #[inline]
    pub fn local(&self) -> &Tm {
        &self.local
    }
    #[inline]
    pub fn ls(&self) -> &str {
        self.ls.as_str()
    }
    #[inline]
    pub fn utc(&self) -> &Tm {
        &self.utc
    }
    #[inline]
    pub fn us(&self) -> &str {
        self.us.as_str()
    }
}

// stat  .gitignore
#[test]
fn main() {
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
