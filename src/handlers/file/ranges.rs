use rand::random;

use std::{
    mem,
    ops::{Index, IndexMut},
    str::FromStr,
};

// [s, e, partial-header)
pub type Range = (u64, u64, String);
pub type Ranges = Vec<Range>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RangesForm {
    partail: bool,
    boundary: u128,
    ranges: Ranges,
    content_type: String,
    offset: usize,
}

impl RangesForm {
    pub fn is_partail(&self) -> bool {
        self.partail
    }
    pub fn boundary_eof(&mut self) -> Option<String> {
        if self.boundary > 0 {
            let eof = format!("\r\n--{:#034x}--", self.boundary);
            self.boundary = 0;
            return Some(eof);
        }

        None
    }

    // (content's len) -> (Content-Length, Range)
    pub fn build(&mut self, len: u64, typo: &mut String) -> Result<(u64, String), &'static str> {
        fn rangefc(p: &mut (u64, u64, String), len: u64) -> Result<(), &'static str> {
            if p.1 == u64::max_value() && len > 0 {
                p.1 = len - 1;
            }
            if p.1 > len {
                Err("pfc's range.1 > len")
            } else {
                Ok(())
            }
        }

        let mut content_length = 0;
        let mut content_range = String::new();

        match self.ranges.len() {
            0 => unreachable!("empty RangesValue"),
            1 => {
                // content-length: 101
                // content-range: bytes 0-100/1807
                let p = &mut self.ranges[0usize];

                rangefc(p, len)?;
                content_range = format!("bytes {}-{}/{}", p.0, p.1, len);
                content_length = p.1 + 1 - p.0;
                // fix for 206 [start, end]
                if p.1 < len {
                    p.1 += 1;
                }
            }
            more => {
                // HTTP/1.1 206 Partial Content
                // Date: Wed, 15 Nov 2015 06:25:24 GMT
                // Last-Modified: Wed, 15 Nov 2015 04:58:08 GMT
                // Content-Length: 1741
                // Content-Type: multipart/byteranges; boundary=String_separator

                // --String_separator
                // Content-Type: application/pdf
                // Content-Range: bytes 234-639/8000

                // ...the first range...
                // --String_separator
                // Content-Type: application/pdf
                // Content-Range: bytes 4590-7999/8000

                // ...the second range
                // --String_separator--
                self.boundary = random();
                if self.boundary == 0 {
                    self.boundary += 1;
                }

                self.content_type = format!("multipart/byteranges; boundary={:#034x}", self.boundary);
                mem::swap(&mut self.content_type, typo);

                for idx in 0..more {
                    let p = &mut self.ranges[idx];
                    rangefc(p, len)?;
                    p.2 = format!(
                        "--{:#034x}\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\n",
                        self.boundary, typo, p.0, p.1, len
                    );
                    content_length += p.1 + 1 - p.0 + p.2.len() as u64;

                    // fix for 206 [start, end]
                    if p.1 < len {
                        p.1 += 1;
                    }
                }

                content_length += 2 + 4 + 34;
            }
        }

        Ok((content_length, content_range))
    }
}

impl FromStr for RangesForm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("bytes=") {
            return Err(r#"!s.starts_with("bytes=")"#);
        }

        let s = &s["bytes=".len()..];

        fn parse_pair(s: &str) -> Result<(u64, u64, String), &'static str> {
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

            Ok((start, end, String::new()))
        }

        let mut ranges = vec![];

        if s.contains(',') {
            for p in s.split(',') {
                parse_pair(p).map(|p| ranges.push(p))?;
            }
        } else {
            parse_pair(s).map(|p| ranges.push(p))?;
        }

        Ok(ranges.into())
    }
}

impl Into<RangesForm> for Ranges {
    fn into(self) -> RangesForm {
        RangesForm {
            content_type: String::new(),
            partail: true,
            ranges: self,
            boundary: 0,
            offset: 0,
        }
    }
}

impl Into<RangesForm> for u64 {
    fn into(self) -> RangesForm {
        RangesForm {
            offset: 0,
            boundary: 0,
            partail: false,
            content_type: String::new(),
            ranges: vec![(0, self, String::new())],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangesResp<C> {
    pub form: RangesForm,
    pub content: C,
}

impl<C> RangesResp<C> {
    pub fn new(form: RangesForm, content: C) -> Self {
        Self { form, content }
    }
    pub fn set_offset(&mut self, offset: usize) {
        debug_assert!(self.form.offset + offset <= self.form.ranges.len());
        self.form.offset += offset;
    }
    pub fn is_empty(&self) -> bool {
        self.form.ranges.len() < self.form.offset + 1
    }
}

impl Index<usize> for RangesForm {
    type Output = Range;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.ranges[self.offset + idx]
    }
}

impl IndexMut<usize> for RangesForm {
    fn index_mut(&mut self, idx: usize) -> &mut Range {
        &mut self.ranges[self.offset + idx]
    }
}
