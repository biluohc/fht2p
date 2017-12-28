use url::percent_encoding::percent_encode_byte;
use time::{self, Duration, Tm};
use hyper::server::Request;
use hyper_fs::Config;

use consts;

use std::fs::{self, DirEntry, FileType};
use std::path::{Path, PathBuf};
use std::cmp::Ordering;
use std::fmt;
use std::env;
use std::io;

pub fn render_html(title: &str, index: &PathBuf, req: &Request, order: &EntryOrder, config: &Config) -> io::Result<String> {
    let metadatas = EntryMetadata::read_dir(
        index,
        config.get_follow_links(),
        config.get_hide_entry(),
        order,
    )?;
    let next_order = order.next();
    let remote_addr = req.remote_addr().unwrap();
    let mut html = format!(
        "<!DOCTYPE html><html><head>
<meta charset=\"UTF-8\">
<style type=\"text/css\">{}</style> 
<title>{}</title></head>
<body><h1><span id=\"client\">{}{}:{}{}</span><a href=\"{}../\">{}</a><p></h1>
<table><thead><tr style=\"border-bottom: 0.1px solid #000080;\">
<th><button onclick=\"javascrtpt:window.location.href='?Sort={}'\">Name</button></th>
<th><button onclick=\"javascrtpt:window.location.href='?Sort={}'\">Last_modified</button></th>
<th><button onclick=\"javascrtpt:window.location.href='?Sort={}'\">Size</button></th>
</tr></thead><tbody>
",
        consts::CSS,
        title,
        consts::SPACEHOLDER.repeat(8),
        remote_addr.ip(),
        remote_addr.port(),
        consts::SPACEHOLDER.repeat(8),
        req.path(),
        title,
        next_order.0,
        next_order.1,
        next_order.2
    );
    metadatas.iter().for_each(|md| html.push_str(&md.format()));

    let tail = format!(
        "</tbody></table></body><address><a href=\"{}\">{}/{}</a>({}/{}) server at <a href=\"/\">{}:{}</a></address></html>",
        consts::URL,
        consts::NAME,
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH,
        consts::SERVER_ADDR.get().ip(),
        consts::SERVER_ADDR.get().port(),
    );
    html.push_str(&tail);
    Ok(html)
}

pub struct EntryMetadata {
    pub name: String,
    pub size: Option<u64>,
    pub modified: Option<Tm>,
    pub typo: Option<FileType>,
}

impl EntryMetadata {
    pub fn new(d: &DirEntry, follow_links: bool, hide_entry: bool) -> Option<Self> {
        let name = d.file_name().to_string_lossy().into_owned().to_owned();
        if hide_entry && name.starts_with('.') {
            return None;
        }
        let metadata = d.metadata().ok();
        let typo = metadata.as_ref().map(|md| md.file_type());
        if !follow_links && typo.as_ref().map(|t| t.is_symlink()).unwrap_or(true) {
            return None;
        }
        Some(Self {
            name: name,
            size: metadata.as_ref().map(|md| md.len()),
            typo: typo,
            modified: metadata.as_ref().and_then(|md| {
                md.modified()
                    .ok()
                    .and_then(|mt| mt.elapsed().ok())
                    .and_then(|sd| Duration::from_std(sd).ok())
                    .map(|du| time::now() - du)
            }),
        })
    }
    pub fn read_dir<P: AsRef<Path>>(dir: P, follow_links: bool, hide_entry: bool, order: &EntryOrder) -> io::Result<Vec<Self>> {
        let entries = fs::read_dir(dir)?;
        let mut entries_vec = Vec::new();
        // let mut name_len_max = 0;
        entries.into_iter().filter_map(|e| e.ok()).for_each(|e| {
            if let Some(d) = EntryMetadata::new(&e, follow_links, hide_entry) {
                entries_vec.push(d)
            }
        });
        order.sort(&mut entries_vec);
        Ok(entries_vec)
    }
    pub fn format(&self) -> String {
        let name_enc = self.name
            .bytes()
            .map(percent_encode_byte)
            .collect::<String>();
        let (name_style, name_enc_tail, name_tail) = self.typo
            .as_ref()
            .map(|ft| match (ft.is_dir(), ft.is_symlink()) {
                (false, false) => ("", "", ""),
                (true, false) => (" class=\"dir\"", "/", "/"),
                (false, true) => (" class=\"symlink\"", "", "@"),
                // unreachable!() ?
                (true, true) => (" class=\"dir symlink\"", "/", "/@"),
            })
            .unwrap_or(("", "", ""));

        format!(
            "<tr><td{}><a href=\"{}{}\">{}{}</a></td><td>{}{}</td><td>{}<b>{}</b></td></tr>\n",
            name_style,
            name_enc,
            name_enc_tail,
            self.name,
            name_tail,
            consts::SPACEHOLDER.repeat(3),
            mtime_humman(&self.modified),
            consts::SPACEHOLDER.repeat(3),
            size_human(&self.size)
        )
    }
}

pub fn size_human(size: &Option<u64>) -> String {
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
        .unwrap_or_else(|| "--".to_owned())
}

pub fn mtime_humman(mtime: &Option<Tm>) -> String {
    mtime
        .as_ref()
        .and_then(|mt| mt.strftime("%Y-%m%d&nbsp;&nbsp;%I:%M:%S").ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "-- -".to_owned())
}

#[derive(Debug)]
pub enum EntryOrder {
    /// if use None, conflicts with Option::None,
    Empty,
    Name,
    NameRev,
    Size,
    SizeRev,
    Modified,
    ModifiedRev,
}

impl EntryOrder {
    pub fn new(req_query: Option<&str>) -> Self {
        use self::EntryOrder::*;
        match req_query {
            None => Empty,
            Some(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "sort=name" => Name,
                    "sort=namerev" => NameRev,
                    "sort=size" => Size,
                    "sort=sizerev" => SizeRev,
                    "sort=modified" => Modified,
                    "sort=modifiedrev" => ModifiedRev,
                    _ => Empty,
                }
            }
        }
    }
    pub fn next(&self) -> (&'static str, &'static str, &'static str) {
        use self::EntryOrder::*;
        match *self {
            Empty | NameRev | ModifiedRev | SizeRev => ("Name", "Modified", "Size"),
            Name => ("NameRev", "Modified", "Size"),
            Size => ("Name", "Modified", "SizeRev"),
            Modified => ("Name", "ModifiedRev", "Size"),
        }
    }
    pub fn sort(&self, entries: &mut Vec<EntryMetadata>) {
        use self::EntryOrder::*;
        match *self {
            Empty => {}
            Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
            NameRev => entries.sort_by(|b, a| a.name.cmp(&b.name)),
            Size => entries.sort_by(|b, a| match (a.size.as_ref(), b.size.as_ref()) {
                (Some(aa), Some(bb)) => aa.cmp(bb),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                _ => Ordering::Equal,
            }),
            SizeRev => entries.sort_by(|a, b| match (a.size.as_ref(), b.size.as_ref()) {
                (Some(aa), Some(bb)) => aa.cmp(bb),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                _ => Ordering::Equal,
            }),
            Modified => entries.sort_by(|a, b| match (a.modified.as_ref(), b.modified.as_ref()) {
                (Some(aa), Some(bb)) => aa.cmp(bb),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                _ => Ordering::Equal,
            }),
            ModifiedRev => entries.sort_by(|b, a| match (a.modified.as_ref(), b.modified.as_ref()) {
                (Some(aa), Some(bb)) => aa.cmp(bb),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                _ => Ordering::Equal,
            }),
        }
    }
}

impl fmt::Display for EntryOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::EntryOrder::*;
        f.write_str(match *self {
            Empty => "Empty",
            Name => "Name",
            NameRev => "NameRev",
            Size => "Size",
            SizeRev => "SizeRev",
            Modified => "Modified",
            ModifiedRev => "ModifiedRev",
        })
    }
}
