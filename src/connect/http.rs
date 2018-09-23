use bytes::BytesMut;
use chrono::offset::Local;
use http::header::HeaderValue;
use http::{self, Request, Response};
use httparse;
use tokio::codec::{Decoder, Encoder};

use std::{fmt, io};

pub struct Http;

impl Encoder for Http {
    type Item = Response<&'static str>;
    type Error = io::Error;

    fn encode(&mut self, resp: Response<&'static str>, dst: &mut BytesMut) -> io::Result<()> {
        use std::fmt::Write;

        let now = Local::now().naive_local();

        // 根据 200状态码 手动转字符串, 其它的不用管
        let status = if resp.status().as_u16() == 200 {
            "200 Connection Established".to_owned()
        } else {
            resp.status().to_string()
        };
        write!(
            BytesWrite(dst),
            "\
             HTTP/1.1 {}\r\n\
             Content-Length: {}\r\n\
             Date: {}\r\n\
             ",
            status,
            resp.body().len(),
            now.format("%Y-%m-%dT%H:%M:%S.%3f")
        ).unwrap();

        for (k, v) in resp.headers() {
            dst.extend_from_slice(k.as_str().as_bytes());
            dst.extend_from_slice(b": ");
            dst.extend_from_slice(v.as_bytes());
            dst.extend_from_slice(b"\r\n");
        }

        dst.extend_from_slice(b"\r\n");
        dst.extend_from_slice(resp.body().as_bytes());

        return Ok(());
    }
}

impl Decoder for Http {
    type Item = Request<()>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Request<()>>> {
        // Don’t need todo: we should grow this headers array if parsing fails and asks for more headers
        let mut headers = [None; 16];
        let (method, path, version, amt) = {
            let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut parsed_headers);
            let status = r.parse(src).map_err(|e| {
                let msg = format!("failed to parse http request: {:?}", e);
                io::Error::new(io::ErrorKind::Other, msg)
            })?;

            let amt = match status {
                httparse::Status::Complete(amt) => amt,
                httparse::Status::Partial => return Ok(None),
            };

            let toslice = |a: &[u8]| {
                let start = a.as_ptr() as usize - src.as_ptr() as usize;
                assert!(start < src.len());
                (start, start + a.len())
            };

            for (i, header) in r.headers.iter().enumerate() {
                let k = toslice(header.name.as_bytes());
                let v = toslice(header.value);
                headers[i] = Some((k, v));
            }

            (
                toslice(r.method.unwrap().as_bytes()),
                toslice(r.path.unwrap().as_bytes()),
                r.version.unwrap(),
                amt,
            )
        };
        if version != 1 {
            return Err(io::Error::new(io::ErrorKind::Other, "only HTTP/1.1 accepted"));
        }
        let data = src.split_to(amt).freeze();
        let mut ret = Request::builder();
        ret.method(&data[method.0..method.1]);
        ret.uri(data.slice(path.0, path.1));
        ret.version(http::Version::HTTP_11);
        for header in headers.iter() {
            let (k, v) = match *header {
                Some((ref k, ref v)) => (k, v),
                None => break,
            };
            let value = unsafe { HeaderValue::from_shared_unchecked(data.slice(v.0, v.1)) };
            ret.header(&data[k.0..k.1], value);
        }

        let req = ret.body(()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Some(req))
    }
}

pub struct BytesWrite<'a>(&'a mut BytesMut);

impl<'a> fmt::Write for BytesWrite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::write(self, args)
    }
}
