use std::io::{BufRead, BufReader};
use std::ffi::OsStr;
use std::path::Path;
use std::fs::File;
use std::str;

/// `utf-8`
pub const DOC_CHARSET: &'static str = ";charset=utf-8";

macro_rules! doc_add {
     ($($key: expr => $val: expr),+) => (
            /// `"html" => "text/html"`        
            #[derive(Debug,PartialEq)]
            pub struct Doc(&'static str, &'static str);
            impl Doc {
                pub fn new<S: AsRef<str>>(extension: S)->Self {
                    match extension.as_ref() {
                        $($key =>Doc($key,$val)),+
                        , _ =>Doc("_","text/plain")
                    }
                }
            }
    );
}
impl Doc {
    // `html`
    pub fn extension(&self) -> &'static str {
        self.0
    }
    // `text/html`
    pub fn as_str(&self) -> &'static str {
        self.1
    }
    // `text/html`
    pub fn to_string(&self) -> String {
        self.1.to_owned()
    }
}
doc_add! (
"*"=>"text/plain",
"css"=>"text/css",
"js"=>"text/javascript",
"json"=>"application/json ",
"htm"=>"text/html",
"html"=>"text/html",
"xhtml"=>"text/html",
"xml"=>"application/xml",
"svg"=>"text/xml",
"m3u"=>"audio/mpegurl",
"m3u8"=>"application/x-mpegURL");

macro_rules! bin_add {
     ($($key: expr => $val: expr),+) => (
            /// `"mp4" => "video/mp4"`        
            #[derive(Debug,PartialEq)]
            pub struct Bin(&'static str, &'static str);
            impl Bin {
                pub fn new<S: AsRef<str>>(extension: S)->Self {
                    match extension.as_ref() {
                        $($key =>Bin($key,$val)),+
                        , _ =>Bin("_","application/octet-stream")
                    }
                }
            }
    );
}
impl Bin {
    // `mp4`
    pub fn extension(&self) -> &'static str {
        self.0
    }
    // `video/mp4`
    pub fn as_str(&self) -> &'static str {
        self.1
    }
    // `video/mp4`
    pub fn to_string(&self) -> String {
        self.1.to_owned()
    }
}
bin_add! (
"*"=>"application/octet-stream",
"ps"=>"postscript",
"pdf"=>"application/pdf",
"xls"=>"application/vnd.ms-excel",
"doc"=>"application/msword",
"ppt"=>"application/vnd.ms-powerpoint",
"ico"=>"image/x-icon",
"jpg"=>"image/jpeg",
"jpeg"=>"image/jpeg",
"png"=>"image/png",
"apng"=>"image/png",
"webp"=>"image/webp",
"midi"=>"audio/mid",
"mid"=>"audio/mid",
"aif"=>"audio/aiff",
"aiff"=>"audio/aiff",
"flac"=>"audio/flac",
"mp2"=>"audio/mp2",
"mp3"=>"audio/mp3",
"ogg"=>"audio/ogg",
"aac"=>"audio/aac",
"wav"=>"audio/wav",
"wma"=>"audio/x-ms-wma",
"avi"=>"video/avi",
"3gp"=>"video/3gpp",
"ts"=>"video/MP2T",
"mp4"=>"video/mp4",
"mpg"=>"video/mpg",
"mpeg"=>"video/mpg",
"webm"=>"video/webm",
"mkv"=>"video/x-matroska",
"wmv"=>"video/x-ms-wmv",
"mov"=>"video/quicktime",
"swf"=>"application/x-shockwave-flash",
"flv"=>"video/x-flv",
"7z"=>"application/x-7z-compressed",
"zip"=>"application/zip",
"gzip"=>"application/gzip",
"rar"=>"application/x-rar-compressed",
"iso"=>"application/iso-image"
);

/// `ContentType`
#[derive(Debug,PartialEq)]
pub enum ContentType {
    Doc(Doc),
    Bin(Bin),
}

impl ContentType {
    pub fn extension(&self) -> &'static str {
        match *self {
            ContentType::Doc(ref d) => d.extension(),
            ContentType::Bin(ref b) => b.extension(),
        }
    }
    /// `Doc` without `CHARSET`
    pub fn as_str(&self) -> &'static str {
        match *self {
            ContentType::Doc(ref d) => d.as_str(),
            ContentType::Bin(ref b) => b.as_str(),
        }
    }
    /// `Doc` with `CHARSET: utf8`
    pub fn to_string(&self) -> String {
        match *self {
            ContentType::Doc(ref d) => d.to_string() + DOC_CHARSET,
            ContentType::Bin(ref b) => b.to_string(),
        }
    }
    pub fn from_bytes<P: AsRef<Path>>(path: P, bytes: &[u8]) -> Self {
        if str::from_utf8(bytes).is_ok() {
            Self::parse_doc(path)
        } else {
            Self::parse_bin(path)
        }
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(&path).unwrap();
        let mut file = BufReader::new(file);
        if file.read_line(&mut String::new()).is_ok() {
            Self::parse_doc(path)
        } else {
            Self::parse_bin(path)
        }
    }
    fn extension_get<P: AsRef<Path>>(path: P) -> String {
        path.as_ref()
            .extension()
            .unwrap_or_else(|| OsStr::new("*"))
            .to_string_lossy()
            .into_owned()
    }
    pub fn parse_doc<P: AsRef<Path>>(path: P) -> Self {
        let extension = Self::extension_get(path);
        let extension = extension.as_str();
        ContentType::Doc(Doc::new(extension))
    }
    pub fn parse_bin<P: AsRef<Path>>(path: P) -> Self {
        let extension = Self::extension_get(path);
        let extension = extension.as_str();
        ContentType::Bin(Bin::new(extension))
    }
    pub fn html() -> Self {
        ContentType::Doc(Doc::new("html"))
    }
}
