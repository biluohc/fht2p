use percent_encoding::{percent_decode, percent_encode, AsciiSet, PercentEncode, CONTROLS};
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

pub fn url_component_encode(input: &str) -> PercentEncode {
    percent_encode(input.as_bytes(), PATH_ENCODE_SET)
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
    ]
    .into_iter()
    .for_each(|(i, o)| assert_eq!(o.to_string(), url_for_parent(i)))
}

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
/*
for i in 0:127
    @printf(".add(0x%02x) // %d %s\n", i, i, Char(i))
end
*/
const PATH_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(0x00) // 0
    .add(0x01) // 1
    .add(0x02) // 2
    .add(0x03) // 3
    .add(0x04) // 4
    .add(0x05) // 5
    .add(0x06) // 6
    .add(0x07) // 7
    .add(0x08) // 8
    .add(0x09) // 9
    .add(0x0a) // 10
    .add(0x0b) // 11
    .add(0x0c) // 12
    .add(0x0d) // 13
    .add(0x0e) // 14
    .add(0x0f) // 15
    .add(0x10) // 16
    .add(0x11) // 17
    .add(0x12) // 18
    .add(0x13) // 19
    .add(0x14) // 20
    .add(0x15) // 21
    .add(0x16) // 22
    .add(0x17) // 23
    .add(0x18) // 24
    .add(0x19) // 25
    .add(0x1a) // 26
    .add(0x1b) // 27
    .add(0x1c) // 28
    .add(0x1d) // 29
    .add(0x1e) // 30
    .add(0x1f) // 31
    .add(0x20) // 32
    // .add(0x21) // 33 !
    .add(0x22) // 34 "
    .add(0x23) // 35 #
    // .add(0x24) // 36 $
    .add(0x25) // 37 %
    // .add(0x26) // 38 &
    // .add(0x27) // 39 '
    // .add(0x28) // 40 (
    // .add(0x29) // 41 )
    // .add(0x2a) // 42 *
    // .add(0x2b) // 43 +
    // .add(0x2c) // 44 ,
    // .add(0x2d) // 45 -
    // .add(0x2e) // 46 .
    // .add(0x2f) // 47 /
    // .add(0x30) // 48 0
    // .add(0x31) // 49 1
    // .add(0x32) // 50 2
    // .add(0x33) // 51 3
    // .add(0x34) // 52 4
    // .add(0x35) // 53 5
    // .add(0x36) // 54 6
    // .add(0x37) // 55 7
    // .add(0x38) // 56 8
    // .add(0x39) // 57 9
    // .add(0x3a) // 58 :
    // .add(0x3b) // 59 ;
    // .add(0x3c) // 60 <
    // .add(0x3d) // 61 =
    // .add(0x3e) // 62 >
    .add(0x3f) // 63 ?
    // .add(0x40) // 64 @
    // .add(0x41) // 65 A
    // .add(0x42) // 66 B
    // .add(0x43) // 67 C
    // .add(0x44) // 68 D
    // .add(0x45) // 69 E
    // .add(0x46) // 70 F
    // .add(0x47) // 71 G
    // .add(0x48) // 72 H
    // .add(0x49) // 73 I
    // .add(0x4a) // 74 J
    // .add(0x4b) // 75 K
    // .add(0x4c) // 76 L
    // .add(0x4d) // 77 M
    // .add(0x4e) // 78 N
    // .add(0x4f) // 79 O
    // .add(0x50) // 80 P
    // .add(0x51) // 81 Q
    // .add(0x52) // 82 R
    // .add(0x53) // 83 S
    // .add(0x54) // 84 T
    // .add(0x55) // 85 U
    // .add(0x56) // 86 V
    // .add(0x57) // 87 W
    // .add(0x58) // 88 X
    // .add(0x59) // 89 Y
    // .add(0x5a) // 90 Z
    // .add(0x5b) // 91 [
    // .add(0x5c) // 92 \
    // .add(0x5d) // 93 ]
    // .add(0x5e) // 94 ^
    // .add(0x5f) // 95 _
    // .add(0x60) // 96 `
    // .add(0x61) // 97 a
    // .add(0x62) // 98 b
    // .add(0x63) // 99 c
    // .add(0x64) // 100 d
    // .add(0x65) // 101 e
    // .add(0x66) // 102 f
    // .add(0x67) // 103 g
    // .add(0x68) // 104 h
    // .add(0x69) // 105 i
    // .add(0x6a) // 106 j
    // .add(0x6b) // 107 k
    // .add(0x6c) // 108 l
    // .add(0x6d) // 109 m
    // .add(0x6e) // 110 n
    // .add(0x6f) // 111 o
    // .add(0x70) // 112 p
    // .add(0x71) // 113 q
    // .add(0x72) // 114 r
    // .add(0x73) // 115 s
    // .add(0x74) // 116 t
    // .add(0x75) // 117 u
    // .add(0x76) // 118 v
    // .add(0x77) // 119 w
    // .add(0x78) // 120 x
    // .add(0x79) // 121 y
    // .add(0x7a) // 122 z
    // .add(0x7b) // 123 {
    // .add(0x7c) // 124 |
    // .add(0x7d) // 125 }
    // .add(0x7e) // 126 ~
    .add(0x7f); // 127 
