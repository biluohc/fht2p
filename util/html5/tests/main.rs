extern crate html5;
use html5::{HTML, TagSingle, TagDouble};

#[macro_use]
extern crate stderr;

use std::path::Path;
use std::fs;

const PATH: &'static str = ".";

#[test]
fn main() {
    let mut html = HTML::new();
    errln!("{:?}", html);
    html = html.push(HTML::head()
        .push(TagSingle::new("link").add_attrs(vec![("rel", "stylesheet"), ("type", "text/css"), ("href", "/style.css")].iter()))
        .push(TagDouble::new("script").add_attrs(vec![("type", "text/javascript"), ("src", "/app.js")].iter()))
        .push(TagDouble::new("title").push("颜")));
    errln!("{:?}", html);
    html = dir_to_html(PATH, html);

    let html_fmt = html.to_string();
    errln!("{}", html_fmt);
    let mut html_write: Vec<u8> = Vec::new();
    let mut html_write_all: Vec<u8> = Vec::new();
    let w = html.write(&mut html_write);
    let all = html.write_all(&mut html_write_all);
    errln!("len()/fmt()/write()/write_all(): {}/{}/{:?}/{:?}",
           html.len(),
           html_fmt.len(),
           w,
           all);
    assert_eq!(html.len() == html_fmt.len(), html.len() == w.unwrap());

    errln!("fmt==w/fmt==all/w==all: {}/{}/{}",
           html_fmt.as_bytes() == html_write.as_slice(),
           html_fmt.as_bytes() == html_write_all.as_slice(),
           html_write == html_write_all);
    assert_eq!(html_fmt.as_bytes() == html_write.as_slice(),
               html_fmt.as_bytes() == html_write_all.as_slice());
    assert_eq!(html_fmt.as_bytes() == html_write.as_slice(),
               html_write == html_write_all);

    // diff test
    // {
    //     use std::io::Write;
    //     use std::fs::File;
    //     let mut file = File::create("test_fmt.html").unwrap();
    //     let fmt = file.write(html_fmt.as_bytes());
    //     file.flush().unwrap();
    //     let mut file = File::create("test_write.html").unwrap();
    //     let fw = html.write(&mut file);
    //     file.flush().unwrap();
    //     let mut file = File::create("test_write_all.html").unwrap();
    //     let fall = html.write_all(&mut file);
    //     file.flush().unwrap();
    //     errln!("fmt()/fw(html.write)/fall(html.write_all): {:?}/{:?}/{:?}",
    //            fmt,
    //            fw,
    //            fall);
    // }
}
fn dir_to_html(path: &str, html: HTML) -> HTML {
    let mut ul = TagDouble::new("ul");
    let dir_entrys = fs::read_dir(path).unwrap();
    for entry in dir_entrys {
        let entry = entry.unwrap().path();
        let entry_name = entry.file_name().unwrap().to_string_lossy().into_owned();
        let entry_path = String::new() + path + "/" + &entry_name;
        let (date, size) = fms(&entry_path);
        if Path::new(&entry_path).is_dir() {
            // "/" 区分目录与文件(视觉),并且如果没有它，浏览器不会自动拼路径，这上面坑了好多时间。
            // <li><a href="_i%7E%7Emkv_.html">_i~~mkv_.html</a>      2017-02-16 12:38:02      26.44 K</li>
            ul = ul.push(TagDouble::new("li")
                .push(TagDouble::new("a")
                    .add_attr("href", format!("{}/", entry_path))
                    .push(format!("{}/", entry_name)))
                .push(format!("  {}      {}", date, size)));
        } else {
            ul = ul.push(TagDouble::new("li")
                .push(TagDouble::new("a")
                    .add_attr("href", entry_path)
                    .push(entry_name))
                .push(format!("  {}      {}", date, size)));
        }
    }

    let body = TagDouble::new("body")
        .push(TagDouble::new("pre").push(TagDouble::new("h1")
            .push(format!("{}/", PATH))
            .push(TagDouble::new("span")
                .push("127.0.0.1:58488")
                .add_attr("id", "client"))))
        .push(TagDouble::new("pre")
            .push("Name    Last_modified      Size")
            .push(TagSingle::new("hr"))
            .push(ul))
        .push(TagSingle::new("hr"))
        .push(TagDouble::new("address")
            .push(TagDouble::new("a")
                .add_attr("href", "https://github.com/biluohc/fht2p")
                .push("fht2p"))
            .push("/0.6.1 (linux/x86_64) Server at")
            .push(TagDouble::new("a")
                .add_attr("href", "http://127.0.0.1:8080")
                .push("127.0.0.1:8080")));
    html.push(body)
}

fn fms(path: &str) -> (String, String) {
    if let Ok(metadate) = fs::metadata(path) {
        let len = metadate.len();
        if let Ok(mt) = metadate.modified() {
            let mts = mt.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            return (format!("{}", len), format!("{}", mts));
        }
    }
    ("--".to_owned(), "------".to_owned())
}
