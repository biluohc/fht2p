// crate's info
pub const NAME: &'static str = "fht2p";
pub const VERSION: &'static str = "0.7.0";
pub const AUTHOR: &'static str = "Wspsxing";
pub const EMAIL: &'static str = "biluohc@qq.com";
pub const ABOUT: &'static str = "A HTTP Server for Static File written with Rust";
pub const URL_NAME: &'static str = "Repository";
pub const URL: &'static str = "https://github.com/biluohc/fht2p";

// resource files
pub static FAVICON_ICO: &'static [u8; 4286] = include_bytes!("../../../config/default/favicon.ico");
pub const FAVICON_ICO_PATH: &'static str = "/favicon.ico";
pub static CSS: &'static str = include_str!("../../../config/default/fht2p.css");
pub const CSS_PATH: &'static str = "/fht2p.css";
pub static JS: &'static str = include_str!("../../../config/default/fht2p.js");
pub const JS_PATH: &'static str = "/fht2p.js";

// config file
pub const CONFIG_DEFAULT_PATH: &'static str = "fht2p.ini";
pub const CONFIG_DEFAULT: &'static str = include_str!("../../../config/fht2p.ini");

pub const BUFFER_SIZE: usize = 1024 * 1024; //字节1024*1024=>1m
pub const TIME_OUT: u64 = 5; // 5 secs 以后放到选项/配置
// set_nonblocking 不能使用,因为读取文件会阻塞，只能set_write/read_timeout() 来断开一直阻塞的连接。

#[cfg(windows)]
#[path = "win_ico.rs"]
mod win_ico;
// exe的图标。非Windows平台不需要rc资源。

// codes number strs
pub static CNS: [(u32, &'static str); 45] = [(100, "Continue"),
                                             (101, "Switching Protocols"),
                                             (102, "Processing"),
                                             (118, "Connection timed out"),
                                             (200, "OK"),
                                             (201, "Created"),
                                             (202, "Accepted"),
                                             (203, "Non-Authoritative Information"),
                                             (204, "No Content"),
                                             (205, "Reset Content"),
                                             (206, "Partial Content"),
                                             (207, "Multi-Status"),
                                             (210, "Content Different"),
                                             (300, "Multiple Choices"),
                                             (301, "Moved Permanently"),
                                             (302, "Found"),
                                             (303, "See Other"),
                                             (304, "Not Modified"),
                                             (305, "Use Proxy"),
                                             (307, "Temporary Redirect"),
                                             (400, "Bad Request"),
                                             (401, "Unauthorized"),
                                             (402, "Payment Required"),
                                             (403, "Forbidden"),
                                             (404, "Not Found"),
                                             (405, "Method Not Allowed"),
                                             (406, "Not Acceptable"),
                                             (407, "Proxy Authentication Required"),
                                             (408, "Request Time-out"),
                                             (409, "Conflict"),
                                             (410, "Gone"),
                                             (411, "Length Required"),
                                             (412, "Precondition Failed"),
                                             (413, "Request Entity Too Large"),
                                             (414, "Reques-URI Too Large"),
                                             (415, "Unsupported Media Type"),
                                             (416, "Request range not satisfiable"),
                                             (417, "Expectation Failed"),
                                             (500, "Internal Server Error"),
                                             (501, "Not Implemented"),
                                             (502, "Bad Gateway"),
                                             (503, "Service Unavailable"),
                                             (504, "Gateway Time-out"),
                                             (505, "HTTP Version not supported"),
                                             (0, "Unknown")];

// Content-Types
// https://www.sitepoint.com/web-foundations/mime-types-complete-list/
pub static ETS_DOC: [(&'static str, &'static str); 11] = [("*", "text/plain"),
                                                          ("css", "text/css"),
                                                          ("js", "text/javascript"),
                                                          ("json", "application/json "),
                                                          ("htm", "text/html"),
                                                          ("html", "text/html"),
                                                          ("xhtml", "text/html"),
                                                          ("xml", "application/xml"),
                                                          ("svg", "text/xml"),
                                                          ("m3u", "audio/mpegurl"),
                                                          ("m3u8", "application/x-mpegURL")];

pub static ETS_BIN: [(&'static str, &'static str); 40] = [("*", "application/octet-stream"),
                                                          ("ps", "postscript"),
                                                          ("pdf", "application/pdf"),
                                                          ("xls", "application/vnd.ms-excel"),
                                                          ("doc", "application/msword"),
                                                          ("ppt", "application/vnd.ms-powerpoint"),
                                                          ("ico", "image/x-icon"),
                                                          ("jpg", "image/jpeg"),
                                                          ("jpeg", "image/jpeg"),
                                                          ("png", "image/png"),
                                                          ("apng", "image/png"),
                                                          ("webp", "image/webp"),
                                                          ("midi", "audio/mid"),
                                                          ("mid", "audio/mid"),
                                                          ("aif", "audio/aiff"),
                                                          ("aiff", "audio/aiff"),
                                                          ("flac", "audio/flac"),
                                                          ("mp2", "audio/mp2"),
                                                          ("mp3", "audio/mp3"),
                                                          ("ogg", "audio/ogg"),
                                                          ("aac", "audio/aac"),
                                                          ("wav", "audio/wav"),
                                                          ("wma", "audio/x-ms-wma"),
                                                          ("avi", "video/avi"),
                                                          ("3gp", "video/3gpp"),
                                                          ("ts", "video/MP2T"),
                                                          ("mp4", "video/mp4"),
                                                          ("mpg", "video/mpg"),
                                                          ("mpeg", "video/mpg"),
                                                          ("webm", "video/webm"),
                                                          ("mkv", "video/x-matroska"),
                                                          ("wmv", "video/x-ms-wmv"),
                                                          ("mov", "video/quicktime"),
                                                          ("swf", "application/x-shockwave-flash"),
                                                          ("flv", "video/x-flv"),
                                                          ("7z", "application/x-7z-compressed"),
                                                          ("zip", "application/zip"),
                                                          ("gzip", "application/gzip"),
                                                          ("rar", "application/x-rar-compressed"),
                                                          ("iso", "application/iso-image")];
