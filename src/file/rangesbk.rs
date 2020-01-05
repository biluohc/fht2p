use std::{
    ops::{Index, IndexMut},
    str::FromStr,
};

// [s, e)
#[derive(Debug, Clone, PartialEq)]
pub enum RangeValue {
    Multi(Vec<(u64, u64)>),
    Single((u64, u64)),
}

impl RangeValue {
    pub fn len(&self) -> usize {
        use RangeValue::*;

        match self {
            Single(..) => 1,
            Multi(ref vs) => vs.len(),
        }
    }
    pub fn content_length(&self) -> usize {
        use RangeValue::*;

        let sum = match self {
            Single((s, e)) => e - s,
            Multi(ref vs) => vs.iter().map(|(s, e)| e - s).sum(),
        };

        sum as _
    }

    // (content's len) -> (Content-Length, Content-Range)
    pub fn build(&mut self, len: u64) -> Result<(u64, String), &'static str> {
        use RangeValue::*;

        fn pfc(p: &mut (u64, u64), len: u64) -> Result<(), &'static str> {
            if p.1 == u64::max_value() {
                p.1 = len;
            }
            if p.1 > len {
                Err("pfc's range.1 > len")
            } else {
                Ok(())
            }
        }

        // Content-Range: bytes 0-1023/146515
        // Content-Length: 1024
        // Content-Range: bytes 0-50/1270
        let mut content_length = 0;
        let mut content_range = "bytes ".to_owned();

        match self {
            Single(ref mut p) => {
                pfc(p, len)?;
                content_length += p.1 - p.0;
                content_range.push_str(&format!("{}/{}", p.0, p.1));
            }
            Multi(ref mut vs) => {
                for idx in 0..vs.len() {
                    pfc(&mut vs[idx], len)?;
                }
            }
        }

        content_range.push_str(&format!("/{}", len));

        Ok((content_length, content_range))
    }
}

impl FromStr for RangeValue {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("bytes=") {
            return Err(r#"!s.starts_with("bytes=")"#);
        }

        let s = &s["bytes=".len()..];

        fn parse_pair(s: &str) -> Result<(u64, u64), &'static str> {
            let mut iters = s.split('-');
            let start = iters.next().ok_or("s.split('-')s failed")?.trim();
            let start = if start.is_empty() {
                0
            } else {
                start.parse::<u64>().map_err(|_| "start.parse::<u64>()s")?
            };

            let end = iters.next().ok_or("s.split('-')e failed")?.trim();
            let end = if end.is_empty() {
                u64::max_value()
            } else {
                end.parse::<u64>().map_err(|_| "start.parse::<u64>()e")?
            };

            if start > end {
                return Err("start >= end");
            }

            Ok((start, end))
        }

        if s.contains(',') {
            let mut multi = vec![];
            for p in s.split(',') {
                parse_pair(p).map(|p| multi.push(p.into()))?;
            }
            Ok(multi.into())
        } else {
            parse_pair(s).map(|p| p.into())
        }
    }
}

impl Index<usize> for RangeValue {
    type Output = (u64, u64);

    fn index(&self, idx: usize) -> &Self::Output {
        use RangeValue::*;

        match self {
            Single(ref v) => {
                if idx == 0 {
                    v
                } else {
                    panic!("RangeValue::Single[idx > 0]: {}", idx)
                }
            }
            Multi(ref vs) => &vs[idx],
        }
    }
}

impl IndexMut<usize> for RangeValue {
    fn index_mut(&mut self, idx: usize) -> &mut (u64, u64) {
        use RangeValue::*;

        match self {
            Single(ref mut v) => {
                if idx == 0 {
                    v
                } else {
                    panic!("RangeValue::Single[idx > 0]: {}", idx)
                }
            }
            Multi(ref mut vs) => &mut vs[idx],
        }
    }
}

impl Into<RangeValue> for Vec<(u64, u64)> {
    fn into(self) -> RangeValue {
        RangeValue::Multi(self)
    }
}

impl Into<RangeValue> for u64 {
    fn into(self) -> RangeValue {
        RangeValue::Single((0, self))
    }
}

impl Into<RangeValue> for (u64, u64) {
    fn into(self) -> RangeValue {
        RangeValue::Single(self)
    }
}

impl Into<Ranges> for RangeValue {
    fn into(self) -> Ranges {
        Ranges { ranges: self, offset: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ranges {
    ranges: RangeValue,
    offset: usize,
}

impl Ranges {
    pub fn set_offset(&mut self, offset: usize) {
        debug_assert!(self.offset + offset <= self.ranges.len());
        self.offset += offset;
    }
    pub fn is_empty(&self) -> bool {
        self.ranges.len() < self.offset + 1
    }
}

impl Index<usize> for Ranges {
    type Output = (u64, u64);

    fn index(&self, idx: usize) -> &Self::Output {
        &self.ranges[self.offset + idx]
    }
}

impl IndexMut<usize> for Ranges {
    fn index_mut(&mut self, idx: usize) -> &mut (u64, u64) {
        &mut self.ranges[self.offset + idx]
    }
}
