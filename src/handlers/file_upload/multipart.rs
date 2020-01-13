use bytes::BytesMut;
use futures::StreamExt;
use hyper::{body::Bytes, header, HeaderMap};

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    str,
};

use crate::{base::Body, how::Error};

#[derive(Debug)]
pub struct MultiPart {
    body: Body,
    buf: BytesMut,
    boundary: String,
    content_lenth: Option<u64>,
    offset: u64,
    eof: bool,
}

impl MultiPart {
    pub fn new(body: Body, headers: &HeaderMap) -> Result<Self, Error> {
        let (contentype, boundary) = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .ok_or(format_err!("without content-type"))
            .and_then(|str| contentype_and_boundary(str).map_err(|e| format_err!(e)))?;

        if contentype.to_lowercase().as_str() != "multipart/form-data" {
            return Err(format_err!("content-type isnot multipart/form-data"));
        }

        let content_lenth = headers
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse().ok());

        Ok(Self {
            body,
            content_lenth,
            boundary: boundary.into(),
            buf: BytesMut::new(),
            offset: 0,
            eof: false,
        })
    }

    // b"--------------------------7adee9ed033d3a54\r\nContent-Disposition: form-data; name=\"filename\"; filename=\"pkg.jl\"\r\nContent-Type: application/octet-stream\r\n\r\n"

    pub async fn next_part<'a>(&'a mut self) -> Option<Result<Part<'a>, Error>> {
        fn is_multi_eof(input: &[u8], boundary: &str) -> bool {
            input.len() >= boundary.len() + 4
                && &input[0..2] == b"--".as_ref()
                && &input[2..2 + boundary.len()] == boundary.as_bytes()
                && &input[2 + boundary.len()..boundary.len() + 4] == b"--"
        }

        loop {
            if !self.eof {
                match self.body.next().await {
                    Some(Ok(buf2)) => {
                        self.offset += buf2.len() as u64;
                        if self.buf.is_empty() {
                            self.buf = buf2.as_ref().into();
                        } else {
                            self.buf.extend_from_slice(&buf2);
                        }
                    }
                    Some(Err(e)) => return Some(Err(e.into())),
                    None => self.eof = true,
                };
            }

            let (remain_bytes, fin, ct) = match filename_and_contentype(self.buf.as_ref(), self.boundary.as_str()) {
                Ok((b, f, c)) => (b.len(), f.to_owned(), c.to_owned()),
                Err(e) => {
                    debug!("{:?}", self.buf);
                    if is_multi_eof(self.buf.as_ref(), self.boundary.as_str()) {
                        return None;
                    }

                    if self.eof || self.buf.len() > 512 {
                        debug!("filename_and_contentype() {}/{:?}: {:?}", self.offset, self.content_lenth, e);
                        return Some(Err(format_err!("unexpect eof or toolarge part-header")));
                    }

                    continue;
                }
            };

            debug!(
                "len: {}, remains: {}. finame: {}, ct: {}",
                self.buf.len(),
                remain_bytes,
                fin,
                ct
            );

            if remain_bytes == 0 {
                self.buf.clear();
            } else {
                let advance_bytes = self.buf.len() - remain_bytes;
                let _consumed = self.buf.split_to(advance_bytes);
                // info!(
                //     "consumed: {}\nremains: {}",
                //     std::str::from_utf8(_consumed.as_ref()).unwrap(),
                //     std::str::from_utf8(self.buf.as_ref()).unwrap()
                // );
            }

            return Some(Ok(Part {
                complete: false,
                multi: self,
                contentype: ct,
                filename: fin,
            }));
        }
    }
}

fn filename_and_contentype<'a>(input: &'a [u8], boundary: &str) -> Result<(&'a [u8], &'a str, &'a str), &'static str> {
    use nom::{
        bytes::complete::{is_not, tag, take_until},
        character::complete::char,
        error::ErrorKind,
        sequence::{delimited, preceded, separated_pair},
        IResult,
    };

    fn fac<'a>(input: &'a [u8], boundary: &str) -> IResult<&'a [u8], &'a [u8], (&'a [u8], ErrorKind)> {
        let headerp = move |inp: &'a [u8]| {
            let (i, _) = tag::<_, _, (&[u8], ErrorKind)>("--")(inp)?;
            // println!("1: {}", str::from_utf8(i).unwrap());

            let (i, o2) = tag::<_, _, (&[u8], ErrorKind)>(boundary)(i)?;
            // println!("2: {}\n{:?}", str::from_utf8(i).unwrap(), i);

            let (i, _) = tag::<_, _, (&[u8], ErrorKind)>("\r\n")(i)?;
            // println!("3: {}", str::from_utf8(i).unwrap());

            Ok((i, o2))
        };

        let crlf2 = |inp: &'a [u8]| {
            const SEP: &str = "\r\n\r\n";

            let (i, o) = take_until::<_, _, (&[u8], ErrorKind)>(SEP)(inp)?;
            // let (i, o) = take::<_, _, (&[u8], ErrorKind)>(SEP.len())(i)?;
            Ok((&i[SEP.len()..], o))
        };

        preceded(headerp, crlf2)(input)
    }

    let (b, header) = fac(input, boundary.as_ref()).map_err(|_e| "Invalid part header")?;

    // debug!("b: {:?}\nheader: {:?}", str::from_utf8(b), str::from_utf8(header));

    let (i, _) = take_until::<_, _, (&[u8], ErrorKind)>("filename=")(header).map_err(|_: nom::Err<_>| "Invalid filename")?;

    let (_i1, (_i2, o)) = separated_pair(tag("filename"), tag("="), delimited(char('"'), is_not("\""), char('"')))(i)
        .map_err(|_: nom::Err<()>| "Invalid filename=")?;

    let filename = str::from_utf8(o).map_err(|_| "invalid str filename")?;

    Ok((b, filename, ""))
}

#[test]
fn filename_and_contentype_test() {
    assert!(filename_and_contentype("--------------------------7adee9ed033d3a54\r\nContent-Disposition: form-data; name=\"filename\"; filename=\"pkg.jl\"\r\nContent-Type: application/octet-stream\r\n\r\n".as_bytes(), 
    "------------------------7adee9ed033d3a54").is_ok())
}

#[derive(Debug)]
pub struct Part<'a> {
    multi: &'a mut MultiPart,
    filename: String,
    contentype: String,
    complete: bool,
}

impl<'a> Part<'a> {
    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }
    pub fn contentype(&self) -> &str {
        self.contentype.as_str()
    }
    pub async fn save(&mut self, path: &Path) -> Result<u64, Error> {
        if path.exists() {
            return Err(format_err!("File already exists"));
        }

        let mut bytesc = 0usize;
        let mut file = File::create(path)?;

        while let Some(chunk) = self.next_chunk().await {
            match chunk.and_then(|chunk| {
                // info!("{}", std::str::from_utf8(bytes.as_ref()).unwrap());
                file.write(chunk.as_ref()).map(|wc| (wc, chunk)).map_err(Error::from)
            }) {
                Ok((wc, chunk)) => {
                    debug_assert_eq!(wc, chunk.len());
                    bytesc += wc;
                }
                Err(e) => {
                    fs::remove_file(path)
                        .map_err(|ee| error!("remove(cause of upload error) {} file {} failed: {:?}", e, path.display(), ee))
                        .ok();
                    return Err(e);
                }
            }
        }

        Ok(bytesc as _)
    }
    pub async fn next_chunk(&mut self) -> Option<Result<Bytes, Error>> {
        if self.complete {
            return None;
        }

        loop {
            let mut last_chunk = std::mem::replace(&mut self.multi.buf, Default::default());

            let mut chunk = if self.multi.eof {
                last_chunk.freeze()
            } else {
                match self.multi.body.next().await {
                    Some(Ok(chunk)) => {
                        // info!("last: {:?}, chunk {}: {:?}", last_chunk, chunk.len(), chunk);
                        self.multi.offset += chunk.len() as u64;
                        if last_chunk.is_empty() {
                            chunk
                        } else {
                            last_chunk.extend_from_slice(chunk.as_ref());
                            last_chunk.freeze()
                        }
                    }
                    None => {
                        self.multi.eof = true;

                        if last_chunk.is_empty() {
                            return None;
                        } else {
                            last_chunk.freeze()
                        }
                    }
                    Some(Err(e)) => return Some(Err(e.into())),
                }
            };

            let res = match parse_part_eof(chunk.as_ref(), &self.multi.boundary) {
                None => Some(Ok(chunk)),
                Some(None) => {
                    if self.complete {
                        return Some(Err(format_err!("unexpected eof")));
                    } else {
                        continue;
                    }
                }
                Some(Some(i)) => {
                    self.complete = true;

                    // remove \r\n from remains
                    let tail = chunk.split_off(i + 2).as_ref().into();
                    self.multi.buf = tail;

                    if i == 0 {
                        None
                    } else {
                        chunk.truncate(chunk.len() - 2);
                        Some(Ok(chunk))
                    }
                }
            };

            return res;
        }
    }
}

#[test]
fn parse_part_eof_test() {
    assert_eq!(parse_part_eof("\r\n--xyz".as_bytes(), "xyz"), Some(Some(0)));
}
fn parse_part_eof(input: &[u8], boundary: &str) -> Option<Option<usize>> {
    let boundary_bytes = boundary.as_bytes();
    let boundary_bytes_size = boundary.as_bytes().len();

    let take = |idx: usize| {
        if idx < 4 {
            b"\r\n--"[idx]
        } else {
            boundary_bytes[idx - 4]
        }
    };

    'for0: for i in 0..input.len() {
        if input[i] == b'\r' {
            let remain_bytes = &input[i..];

            for idx in 0..std::cmp::min(remain_bytes.len(), boundary_bytes_size + 4) {
                let rc = remain_bytes[idx];
                let bc = take(idx);
                if rc != bc {
                    continue 'for0;
                }
            }

            if remain_bytes.len() < boundary_bytes_size {
                return Some(None);
            } else {
                return Some(Some(i));
            }
        }
    }

    None
}

fn contentype_and_boundary(contentype: &str) -> Result<(&str, &str), &'static str> {
    use nom::{
        bytes::complete::{is_not, tag},
        character::complete::multispace0,
        sequence::{preceded, separated_pair},
    };

    let contentype = contentype.trim();

    let parser = separated_pair(
        is_not(";"),
        preceded(tag(";"), multispace0),
        separated_pair(tag("boundary"), tag("="), is_not("; \t\r\n")),
    );

    parser(contentype)
        .map(|(_, (contentype, (_, boundary)))| (contentype.trim(), boundary.trim()))
        .map_err(|_: nom::Err<()>| "invalid content-type or boundary")
}

#[test]
fn contentype_and_boundary_test() {
    assert_eq!(
        contentype_and_boundary("multipart/form-data; boundary=------------------------b7e74a2253ad550d"),
        Ok(("multipart/form-data", "------------------------b7e74a2253ad550d"))
    );

    assert_eq!(
        contentype_and_boundary("multipart/form-data;boundary=------------------------b7e74a2253ad550d"),
        Ok(("multipart/form-data", "------------------------b7e74a2253ad550d"))
    );

    assert_eq!(
        contentype_and_boundary(" multipart/form-data;boundary=------------------------b7e74a2253ad550d "),
        Ok(("multipart/form-data", "------------------------b7e74a2253ad550d"))
    );

    assert!(contentype_and_boundary("multipart/form-data boundary=------------------------b7e74a2253ad550d").is_err());
    assert!(contentype_and_boundary("multipart/form-data; boundary =------------------------b7e74a2253ad550d").is_err());
}
