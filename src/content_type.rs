use hyper::header::Headers;
use hyper::Request;
use mime_guess;

use consts::{MutStatic, CHARSET, CONTENT_TYPE, MAGIC_LIMIT};

use std::io::{self, Read, Seek, SeekFrom};
use std::fs::{File, Metadata};
use std::path::PathBuf;
use std::{mem, str};

/**
`Content-Type`

[MDN.`Complete_list_of_MIME_types`](https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Complete_list_of_MIME_types)

[iana.media-types.xhtml#examples](https://www.iana.org/assignments/media-types/media-types.xhtml#examples)

[`mime_guess`](https://github.com/abonander/mime_guess/blob/master/src/mime_types.rs)
*/
pub fn headers_maker(file: &mut File, metadata: &Metadata, path: &PathBuf, _req: &Request, headers: &mut Headers) -> io::Result<()> {
    if let Some(mime) = mime_guess::guess_mime_type_opt(path) {
        let mime = if mime.type_() == "text" {
            format!("{}/{};{}", mime.type_(), mime.subtype(), CHARSET)
        } else {
            format!("{}/{}", mime.type_(), mime.subtype())
        };
        headers.set_raw(CONTENT_TYPE, mime);
    } else if *MAGIC_LIMIT.get() > metadata.len() {
        let (is_text, _offset) = is_text(file)?;
        if is_text {
            headers.set_raw(CONTENT_TYPE, format!("text/plain; {}", CHARSET));
        } else {
            headers.set_raw(CONTENT_TYPE, "application/octet-stream");
        }
    } else {
        headers.set_raw(CONTENT_TYPE, "application/octet-stream");
    }
    Ok(())
}

/// the length of `tls Buffer`
pub const BUF_LEN: usize = 1024; // fsblock..

thread_local!(
    /// `tls Buffer`
    pub static BUF: MutStatic<[u8;BUF_LEN]>=  MutStatic::new(unsafe {mem::uninitialized()})
);

/// (is utf-8 text, BUF's offset(consider file's magic number in the future?))
pub fn is_text(f: &mut File) -> io::Result<(bool, usize)> {
    fn leading_ones(num: u8) -> u32 {
        (!num).leading_zeros()
    }
    fn inner(f: &mut File, buf: &mut [u8; BUF_LEN]) -> io::Result<(bool, usize)> {
        let len = f.read(buf)?;
        // reset
        f.seek(SeekFrom::Start(0))?;
        match len {
            BUF_LEN => {
                let mut new_char_idx = 0;
                for idx in (0..4).into_iter().rev().map(|i| BUF_LEN - 1 - i) {
                    // ascii is one u8, 0..(0-128),
                    // !ascii is n(2-4) u8: 1(n).., 10..(n-1)
                    let ones = leading_ones(buf[idx]);
                    // println!("{}: {:b}", ones, buf[idx]);
                    if ones != 1 {
                        new_char_idx = idx;
                        break;
                    }
                }
                let res = if new_char_idx != 0 {
                    str::from_utf8(&buf[0..new_char_idx]).is_ok()
                } else {
                    false
                };
                Ok((res, new_char_idx))
            }
            0 => Ok((true, 0)),
            len => Ok((str::from_utf8(&buf[0..len]).is_ok(), len)),
        }
    }
    BUF.with(|buf| inner(f, buf.get_mut()))
}
