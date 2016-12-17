#![allow(dead_code)]

use std::fmt;

pub const NAME: &'static str = "fht2p";
pub const VERSION: f64 = 0.42;
pub const PLATFORM: &'static str = "Linux/openSUSE";

#[derive(Debug)]
pub struct H1 {
    // <pre><h1>/<span>            127.0.0.1:37900</span></h1></pre>
    h1: String,
    addr: String,
}

impl H1 {
    pub fn new(h1: String, addr: String) -> H1 {
        H1 {
            h1: h1,
            addr: addr,
        }
    }
}
impl fmt::Display for H1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "<pre><h1>{}            <span id =\"client\">{}</span></h1></pre>",
               self.h1,
               self.addr)
    }
}

#[derive(Debug)]
pub struct Li {
    // <li><a href="/i~~mkv/">i~~mkv/</a>                             2016-09-23 19:14:52		24.00 K</li>
    link: String,
    name: String,
    date: String,
    size: String,
}
impl Li {
    pub fn new(link: String, name: String, date: String, size: String) -> Li {
        Li {
            link: link,
            name: name,
            date: date,
            size: size,
        }
    }
}

// 这个应该是白写，要对齐条目估计只能在UL的处理时，按每个Li的长度加空格。
impl fmt::Display for Li {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               r#"<li><a href="{}">{}</a>            {}      {}</li>"#,
               self.link,
               self.name,
               self.date,
               self.size,
               )
    }
}
#[derive(Debug)]
pub struct Ul {
    // <pre>  Name     Modified       Size <hr> Vec<Li> </pre><hr>
    // 注意pre标签中间不能有多余换行和空格->大块白板。
    lis: Vec<Li>,
}

impl Ul {
    pub fn new() -> Ul {
        let new: Vec<Li> = Vec::new();
        Ul { lis: new }
    }
    pub fn push(&mut self, li: Li) {
        self.lis.push(li)
    }
}
impl fmt::Display for Ul {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut blank_len_0 = 6usize;
        let blank_len_1 = 6usize;
        let mut li_name_lens: Vec<usize> = vec![];
        for li in &self.lis {
            // 浏览器到底是按字符，字节，还是字宽排版的？ 223 。然而字宽 ?
            let mut str_len = 0;
            for _ in (&li.name).chars() {
                str_len += 1;
            }
            // println!("{}:{}", cv.len(), &li.name);
            li_name_lens.push(str_len);
        }
        let li_name_len_max = (&li_name_lens)
            .into_iter()
            .fold(0usize, |s0, s| if s0 < *s { *s } else { s0 });

        // println!("{}", blank_len_0);
        blank_len_0 += li_name_len_max;
        // println!("{}\n{:?}", blank_len_0, &li_name_lens);
        let mut ui: String = String::new();
        let blanks = |len: usize| {
            let mut str = String::new();
            for _ in 0..len {
                str += " ";
            }
            str
        };
        let namestr = |name: Option<usize>| {
            let bs = blanks(blank_len_0 - name.unwrap());
            bs
        };
        // 加上"<ul>" chrome系不认无序列表，不加火狐不认，艹 。
        ui += &(String::new() + "<pre>" + "Name" + &namestr(Some("Name".len())) +
                "Last modified" + &blanks(blank_len_1) + "Size<hr><ul>");
        let mut li_name_lens_iter = li_name_lens.into_iter();
        for x in &self.lis {
            // r#"<li><a href="{}">{}</a>            {}      {}</li>"#,
            // link: String, name: String, date: String, size: String
            ui += &(String::new() + r#"<li><a href="# + &x.link + r#">"# + &x.name + "</a>" +
                    &namestr(li_name_lens_iter.next()) +
                    &x.date + &blanks(blank_len_1) + &x.size + "</li>");
        }
        // </pre><hr>
        ui += &(String::new() + "</ul></pre><hr>");
        write!(f, "{}", ui)
    }
}

#[derive(Debug)]
pub struct Address {
    // <address> <a href="https://github.com/biluohc/fht2p">fht2p</a>/0.20 (Linux/openSUSE) Server at <a href="http://127.0.0.1:8080">127.0.0.1:8080</a></address>
    // name: &'static str,
    // version: f64
    // platform: &'static str,
    addr: String,
}

impl Address {
    pub fn new(addr: &str) -> Address {
        Address { addr: addr.to_string() }
    }
}
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"<address><a href="https://github.com/biluohc/fht2p">{}</a>/{} ({}) Server at <a href="http://{}">{}</a></address>"#,NAME,VERSION,PLATFORM,self.addr,self.addr)
    }
}

#[derive(Debug)]
pub struct Html {
    title: String,
    h1: H1,
    ul: Option<Ul>,
    address: Address,
}

impl Html {
    pub fn new(t: String, h1: H1, ul: Option<Ul>, addr: Address) -> Html {
        Html {
            title: t,
            h1: h1,
            ul: ul,
            address: addr,
        }
    }
}
impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let html0=r#"<!DOCTYPE html><html><head><meta http-equiv="content-type" content="text/html; charset=UTF-8"><title>"#;
        let html1=r#"</title><link rel="shortcut icon" type="image/x-icon" href="/favicon.ico"><link rel="stylesheet" type="text/css" href="/style.css"></head><body>"#;
        let html3 = "</body></html>";
        let ul = match self.ul.as_ref() {
            Some(s) => format!("{}", s),
            None => "".to_string(),
        };
        write!(f,
               "{}{}{}{}{}{}{}",
               html0,
               self.title,
               html1,
               self.h1,
               ul,
               self.address,
               html3)
    }
}

pub fn s404(client: &str, server: &str) -> String {
    let title = String::from("404 Not Found");
    let addr = Address::new(server);
    let html = Html::new(title.clone(),
                         H1::new(title, client.to_string()),
                         None,
                         addr);
    format!("{}", html)
}

pub fn s500(client: &str, server: &str) -> String {
    let title = String::from("500 Internal Server Error");
    let addr = Address::new(server);
    let html = Html::new(title.clone(),
                         H1::new(title, client.to_string()),
                         None,
                         addr);
    format!("{}", html)
}

fn main() {
    let title = String::from("/.Trash-1000/files");
    let client = "127.0.0.1:41438".to_string();
    let mut ul = Ul::new();
    ul.push(Li::new("/.Trash-1000".to_string(),
                    "../ Parent Directory".to_string(),
                    "2016-12-11 11:46:07".to_string(),
                    "80".to_string()));
    ul.push(Li::new("/.Trash-1000/files/cn_windows_8.1_pro_vl_with_update_x86_dvd_6050910.iso"
                        .to_string(),
                    "cn_windows_8.1_pro_vl_with_update_x86_dvd_6050910.iso".to_string(),
                    "2015-09-08 17:32:19".to_string(),
                    "2.98 G".to_string()));
    ul.push(Li::new("/.Trash-1000/files/Cloud home Page/".to_string(),
                    "Cloud home Page/".to_string(),
                    "2016-11-29 11:16:21".to_string(),
                    "4.00 K".to_string()));
    ul.push(Li::new("/i~~mkv/少司命 - 烟笼长安 by 尉迟嘉馨 to 小烟.f4v".to_string(),
                    "少司命 - 烟笼长安 by 尉迟嘉馨 to 小烟.f4v".to_string(),
                    "2015-03-20 23:58:42".to_string(),
                    "33.86 M".to_string()));
    ul.push(Li::new("/i~~mkv/周深_卡布 - 你曾這樣問過_【さくら_あなたに出会えてよかった_清明櫻花祭_中文填詞】.MP4".to_string(),
                    "周深_卡布 - 你曾這樣問過_【さくら_あなたに出会えてよかった_清明櫻花祭_中文填詞】.MP4".to_string(),
                    "2016-04-23 21:39:22".to_string(),
                    "72.03 M".to_string()));
    ul.push(Li::new("/i~~mkv/桜华月v2.mp4".to_string(),
                    "桜华月v2.mp4".to_string(),
                    "2015-12-15 20:33:42".to_string(),
                    "90.94 M".to_string()));

    let addr = Address::new("127.0.0.1:8080");
    let html = Html::new(title.clone(), H1::new(title, client), Some(ul), addr);
    println!("{}", html);

}
