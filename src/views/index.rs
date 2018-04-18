use url::percent_encoding::percent_encode_byte;
use chrono::format::strftime::StrftimeItems;
use chrono::format::DelayedFormat;
use chrono::offset::Local;
use chrono::{DateTime};
use askama::Template;

use index::model::EntryMetadata;
use super::base::*;

use std::net::SocketAddr;

pub struct Entry<'a> {
    pub name: String,
    pub url: String,
    pub size: Option<String>,
    pub modified: Option<DelayedFormat<StrftimeItems<'a>>>,
    pub class: Option<&'static str>,
}

impl<'a> Entry<'a> {
    fn new(entry: &'a EntryMetadata)->Self {
        let mut url = entry.name.bytes().map(percent_encode_byte).collect::<String>();
        let mut name = entry.name.clone();

        let class = entry.typo.as_ref()
        .and_then(|et| match (et.is_dir(), et.is_symlink()) {
            (false, false) => None,
            (true, false) => {
                 url.push('/');
                 name.push('/');
                 Some("dir")
            },
            (false, true) => {
                 name.push('@');
                 Some("symlink")
            },
            // unreachable!() ?
            (true, true) => {
                 url.push('/');
                 name.push_str("/@");
                 Some("dir symlink")
            },
        });
        let size = size_view( &entry.size );
        let modified = mtime_view( &entry.modified );

        Entry {
            url,
            name,
            size,
            modified,
            class
        }
    }
}

#[derive(Template)]
// #[template(path = "index.html", print = "code", escape= "none")]
#[template(path = "index.html", print = "none", escape= "none")]
pub struct IndexTemplate<'a> {
   _parent: BaseTemplate<'a>,
   entries: Vec<Entry<'a>>,    
   next: (&'static str, &'static str, &'static str),   
}

impl<'a> IndexTemplate<'a>{
    pub fn new(title: &'a str, h1: &'a str, parent: &'a str, client: &'a SocketAddr, next:(&'static str, &'static str, &'static str),   entries: &'a Vec<EntryMetadata>) -> Self {
        IndexTemplate {
            next,            
            _parent: BaseTemplate::new(title, h1, parent, client),
            entries: entries.iter().map(|entry|Entry::new(entry)).collect::<Vec<Entry<'a>>>()
        }
    }
}

pub fn size_view(size: &Option<u64>) -> Option<String> {
    // B，KB，MB，GB，TB，PB，EB，ZB，YB，BB
    static UNITS: &[&'static str] = &["", "K", "M", "G", "T", "P", "E", "Z", "Y", "B"];
    size.as_ref()
        .map(|s| {
            let mut count = 0usize;
            let mut s = *s as f64;
            while s / 1024. > 1. {
                s /= 1024.;
                count += 1;
            }
            format!("{:.02} {}", s, UNITS[count])
        })
}

pub fn mtime_view<'a>(mtime: &'a Option<DateTime<Local>>) -> Option<DelayedFormat<StrftimeItems<'a>>> {
    mtime.as_ref()
    .map(|mt| mt.format("%Y-%m%d&nbsp;&nbsp;%H:%M:%S"))
}