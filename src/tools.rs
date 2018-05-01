use url::percent_encoding::{percent_decode, percent_encode, EncodeSet};

use std::borrow::Cow;

// 以后应该自己组结构体，Hyper 的 Url new方法都没, md 一个 path 都改不了，只能反复 decode..
pub fn url_for_parent(path: &str) -> String {
    let req_path_dec = url_path_decode(path);
    let cow_str = req_path_dec.as_ref();

    let slash_idx = if cow_str.ends_with('/') {
        cow_str[0..cow_str.len() - 1].rfind('/')
    } else {
        cow_str.rfind('/')
    };
    let parent = &cow_str[0..slash_idx.map(|i| i + 1).unwrap_or(1)];

    url_for_path(parent)
}

pub fn url_for_path(path: &str) -> String {
    percent_encode(path.as_bytes(), PATH_ENCODE_SET).to_string()
}

pub fn url_path_decode<'a>(path: &'a str) -> Cow<'a, str> {
    percent_decode(path.as_bytes()).decode_utf8().unwrap()
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct PATH_ENCODE_SET;

impl EncodeSet for PATH_ENCODE_SET {
    #[inline]
    fn contains(&self, byte: u8) -> bool {
        match byte {
            0x00...0x20 => true,
            // #  %  ?
            0x23 | 0x25 | 0x3f => true,
            c if c <= 0x7e => false,
            _ => true,
        }
    }
}

#[test]
fn test_url_for_parent() {
    vec![
        ("/", "/"),
        ("/abc", "/"),
        ("/abc/", "/"),
        ("/abc/def", "/abc/"),
        ("/abc/def/", "/abc/"),
        ("/abc/def/g", "/abc/def/"),
        ("/abc/def/g/", "/abc/def/"),
    ].into_iter()
        .for_each(|(i, o)| assert_eq!(o.to_string(), url_for_parent(i)))
}
