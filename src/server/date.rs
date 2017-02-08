use chrono::*;

#[derive(Debug,Clone,PartialEq)]
pub struct Date {
    local: DateTime<Local>,
    ls: String,
    utc: DateTime<UTC>,
    us: String,
}

impl Date {
    pub fn now() -> Date {
        // 有点不同也无所谓。
        let local = Local::now();
        let utc = UTC::now();
        Date {
            local: local,
            ls: local.format("%Y-%m%d %H:%M:%S").to_string(),
            utc: utc,
            us: utc.format("%a, %e %b %Y %H:%M:%S GMT").to_string(),
        }
    }
    #[inline]
    pub fn as_str(&self) -> &str {
        self.ls.as_ref()
    }
    #[inline]
    pub fn to_http(&self) -> &str {
        self.us.as_ref()
    }
}
impl ToString for Date {
    #[inline]
    fn to_string(&self) -> String {
        self.ls.clone()
    }
}

#[test]
fn main() {
    errln!("#[test]: Date::new()");
    let date = Date::new();
    errln!("Local: {:?}\nLocal: {}", date.local, date.to_str());
    errln!("UTC: {}", date.utc.format("%Y-%m%d %H:%M:%S"));
    errln!("UTC: {:?}\n{}", date.utc, date.to_http());
    // 因为时区原因，不相等。
    // assert_eq!(date.to_string(),
    //            date.utc.format("%Y-%m%d %H:%M:%S").to_string());
    assert!(date.to_string() != date.utc.format("%Y-%m%d %H:%M:%S").to_string());
}
