#![allow(dead_code)]
pub const FAVICON_ICO: &'static [u8; 4286] = include_bytes!("config/default_theme/favicon.ico");
pub const FAVICON_ICO_PATH: &'static str = "/favicon.ico";
pub const HTM_404: &'static str = include_str!("config/default_theme/404.htm");
pub const HTM_500: &'static str = include_str!("config/default_theme/500.htm");
pub const CSS: &'static str = include_str!("config/default_theme/style.css");
pub const CSS_PATH: &'static str = "/style.css";

// const HTM_INDEX: &'static str = include_str!("config/index.htm");
pub const HTM_INDEX_HTML0_TITLE0: &'static str = r#"<!DOCTYPE html><html><head><meta http-equiv="content-type" content="text/html; charset=UTF-8"><title>"#;
pub const HTM_INDEX_TITLE1_H10: &'static str =
    r#"</title><link rel="shortcut icon" type="image/x-icon" href="/favicon.ico">
    <link rel="stylesheet" type="text/css" href="/style.css"></head><body><h1>"#;
pub const HTM_INDEX_H11_SPAN0: &'static str = " <span>            ";
pub const HTM_INDEX_SPAN1_UL0: &'static str =
    r#"</span></h1><pre>Name                             Modified</a>      Size<hr><ul>"#;

// <li><a href="/fht2p.7z">fht2p.7z</a>                             2016_0509		345M</li>
pub const HTM_INDEX_LI0: &'static str = r#"<li><a href=""#;
pub const HTM_INDEX_LI1: &'static str = r#"">"#;
pub const HTM_INDEX_LI2: &'static str = "</a>                             ";
pub const HTM_INDEX_LI3: &'static str = "		";
pub const HTM_INDEX_LI4: &'static str = "</li>";

pub const HTM_INDEX_UL1_ADDR0: &'static str = r#"</ul></pre><hr>
 <address> <a href="https://github.com/biluohc/fht2p">fht2p</a>/0.20 (Linux/openSUSE) Server at "#;

pub const HTM_INDEX_UL1_ADDR00: &'static str = r#"<a href="http://"#;
pub const HTM_INDEX_UL1_ADDR01: &'static str = r#"">"#;
pub const HTM_INDEX_ADDR1_HTML1: &'static str = r#"</a></address></body></html>"#;

pub const CNS: [(u32, &'static str); 3] =
    [(200, "OK"), (404, "Not Found"), (500, "Internal Server Error")];//服务器内部错误,服务器(用户)权限不够

// http://tools.jb51.net/table/http_content_type  后缀-格式
// https://www.sitepoint.com/web-foundations/mime-types-complete-list/
// 常用的应该就这么多，再多应该移到配置文件。
pub const ETS: [(&'static str, &'static str); 52] = [("*", "application/octet-stream"),
                                                     ("txt", "text/plain;charset=utf-8"),
                                                     ("text", "text/plain;charset=utf-8"),
                                                     ("css", "text/css;charset=utf-8"),
                                                     ("js", "text/javascript;charset=utf-8"),
                                                     ("json", "application/json;charset=utf-8"),
                                                     ("htm", "text/html;charset=utf-8"),
                                                     ("html", "text/html;charset=utf-8"),
                                                     ("xhtml", "text/html;charset=utf-8"),
                                                     ("xml", "application/xml;charset=utf-8"),
                                                     ("svg", "text/xml;charset=utf-8"),
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
                                                     ("m3u", "audio/mpegurl"),
                                                     ("m3u8", "application/x-mpegURL"),
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
                                                     ("mp4", "video/mpeg4"),
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
