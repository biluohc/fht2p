extern crate html5;
use html5::{TagSingle, TagDouble, Html};

#[macro_use]
extern crate stderr;

use std::io::Write;
use std::path::Path;
use std::fs::{self, File};
const PATH: &'static str = ".";

#[test]
fn main() {
    let mut html = Html::new();
    errln!("{:?}", html);
    html = html.add_tagd(Html::head().add_tagd(TagDouble::new("title").add_content("颜")));
    errln!("{:?}", html);
    html = dir_to_html(PATH, html);

    let html = html.to_string();
    errln!("Html:\n{}", html);

    let mut file = File::create("main.html").unwrap();
    file.write_all(html.as_bytes()).unwrap();
    file.flush().unwrap();
}
fn dir_to_html(path: &str, html: Html) -> Html {
    let mut ul = TagDouble::new("ul");
    let dir_entrys = fs::read_dir(path).unwrap();
    for entry in dir_entrys {
        let entry = entry.unwrap().path();
        let entry_name = entry.file_name().unwrap().to_string_lossy().into_owned();
        let entry_path = String::new() + path + "/" + &entry_name;
        let (date, size) = fms(&entry_path);
        if Path::new(&entry_path).is_dir() {
            // "/" 区分目录与文件(视觉),并且如果没有它，浏览器不会自动拼路径，这上面坑了好多时间。
            // 仔细对比响应，python3 -m http.server 8000，fuckerfuckf.
            // <li><a href="_i%7E%7Emkv_.html">_i~~mkv_.html</a>      2017-02-16 12:38:02      26.44 K</li>
            ul = ul.add_tagd(TagDouble::new("li")
                .add_tagd(TagDouble::new("a")
                    .add_attr("href", &format!("{}/", entry_path))
                    .add_content(&format!("{}/", entry_name)))
                .add_content(&format!("  {}      {}", date, size)));
        } else {
            ul = ul.add_tagd(TagDouble::new("li")
                .add_tagd(TagDouble::new("a")
                    .add_attr("href", &entry_path)
                    .add_content(&entry_name))
                .add_content(&format!("  {}      {}", date, size)));
        }
    }

    let body = TagDouble::new("body")
        .add_tagd(TagDouble::new("pre").add_tagd(TagDouble::new("h1")
            .add_content(&format!("{}/", PATH))
            .add_tagd(TagDouble::new("span")
                .add_content("127.0.0.1:58488")
                .add_attr("id", "client"))))
        .add_tagd(TagDouble::new("pre")
            .add_content("Name    Last_modified      Size")
            .add_tags(TagSingle::new("hr"))
            .add_tagd(ul))
        .add_tags(TagSingle::new("hr"))
        .add_tagd(TagDouble::new("address")
            .add_tagd(TagDouble::new("a")
                .add_attr("href", "https://github.com/biluohc/fht2p")
                .add_content("fht2p"))
            .add_content("/0.6.1 (linux/x86_64) Server at")
            .add_tagd(TagDouble::new("a")
                .add_attr("href", "http://127.0.0.1:8080")
                .add_content("127.0.0.1:8080")));
    html.add_tagd(body)
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
