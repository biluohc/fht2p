use super::*;
// use super::htm::*;
use time::{Duration, now};
use html5::{HTML, TagSingle, TagDouble};
use std::io::BufReader;
use std::fs::{self, File};
use super::Date;

pub fn header(rc_s: Rc<RcStream>) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("Server".to_owned(), format!("{}/{}", NAME, VERSION));
    if rc_s.keep_alive() {
        map.insert("Connection".to_owned(), "keep-alive".to_owned());
    } else {
        map.insert("Connection".to_owned(), "close".to_owned());
    }
    map
}

pub fn get(rc_s: Rc<RcStream>, mut req: &mut Request) -> Response {
    let map = header(rc_s.clone());
    let path = req.path_rp().to_string();
    let path = Path::new(&path);
    // 路径是断开的链接或不是一般文件/目录(套接字等)或权限不足，文件系统IO错误？
    if path.exists() && path_is_403(path) {
        req.status_set(403);
    }
    // 路径不存在且不是static_files.
    if !path.exists() && !rc_s.arc().sfs().contains_key(req.path_rp().as_str()) {
        req.status_set(404);
    }
    let status_code = req.status;

    match status_code {
        // 正常的文件目录
        200 | 304 if path.is_dir() => dir_to_resp(rc_s.clone(), req, path, map),
        200 | 304 if path.is_file() => file_to_resp(rc_s.clone(), req, path, map),
        // 静态文件
        200 | 304 if !path.exists() => sfs_to_resp(rc_s.clone(), req, path, map),
        // 403的文件目录
        403 if path.exists() => other_status_code_to_resp(rc_s.clone(), req, map),
        // 404的url，文件目录，（注意如果url直接当路径访问，有可能存在.)
        404 => other_status_code_to_resp(rc_s.clone(), req, map),
        // 其它的以后再处理。
        _ => unreachable!(),
    }
}

fn path_is_403(path: &Path) -> bool {
    if path.is_file() {
        File::open(path).is_err()
    } else if path.is_dir() {
        fs::read_dir(path).is_err()
    } else {
        // 套接字，断开的链接，等
        true
    }
}
fn file_to_resp(rc_s: Rc<RcStream>, req: &Request, path: &Path, mut map: HashMap<String, String>) -> Response {
    dbstln!("{}@{} file_to_resp(): {:?}",
            module_path!(),
            rc_s.time().ls(),
            path);
    let ct = file_content_type(rc_s.clone(), path);
    let content = Content::File(File::open(path).unwrap());
    map.insert("Content-Type".to_owned(), ct);
    map.insert("Content-Length".to_owned(), format!("{}", content.len()));

    let code_name = (*rc_s.arc().cns().get(req.status()).unwrap()).to_owned();
    Response::new(req.protocol(),
                  req.version(),
                  *req.status(),
                  code_name,
                  map,
                  content)
}

fn file_content_type(rc_s: Rc<RcStream>, path: &Path) -> String {
    // 那么长的方法调用，好TM气啊。
    use std::ffi::OsStr;
    let exname = path.extension()
        .unwrap_or_else(|| OsStr::new("*"))
        .to_string_lossy()
        .into_owned();
    let exname = exname.as_str();
    let file = File::open(path).unwrap();
    let mut file = BufReader::new(file);
    let mut str = String::new();
    if file.read_line(&mut str).is_ok() {
        //doc
        rc_s.arc().ents_doc(exname)
    } else {
        //bin
        rc_s.arc().ents_bin(exname)
    }
}

fn sfs_to_resp(rc_s: Rc<RcStream>, req: &Request, path: &Path, mut map: HashMap<String, String>) -> Response {
    dbstln!("{}@{} sfs_to_resp(): {:?}",
            module_path!(),
            rc_s.time().ls(),
            path);
    let exname = path.extension().unwrap().to_str().unwrap();
    let doc = rc_s.arc().ents_doc(exname);
    let ct = if doc.as_str() != "text/plain; charset=utf-8" {
        doc
    } else {
        rc_s.arc().ents_bin(exname)
    };

    let content = Content::Sf(*rc_s.arc().sfs().get(req.path_rp().as_str()).unwrap());

    map.insert("Content-Type".to_owned(), ct.to_owned());
    map.insert("Content-Length".to_owned(), format!("{}", content.len()));

    let code_name = rc_s.arc().cns().get(req.status()).unwrap().to_owned();
    Response::new(req.protocol(),
                  req.version(),
                  *req.status(),
                  code_name.to_owned(),
                  map,
                  content)
}

pub fn other_status_code_to_resp(rc_s: Rc<RcStream>, req: &Request, mut map: HashMap<String, String>) -> Response {
    dbstln!("{}@{} other_status_code_to_resp(): {:?}",
            module_path!(),
            rc_s.time().ls(),
            req.status);
    let code_name = (*rc_s.arc().cns().get(req.status()).unwrap()).to_owned();
    let title = format!("{}  {}", req.status(), code_name);
    let h1 = TagDouble::new("h1")
        .push(req.path_raw.clone() + " --> " + &title)
        .push(TagDouble::new("span")
                  .add_attr("id", "client")
                  .push(rc_s.client_addr()));
    let head = htm::head().push(TagDouble::new("title").push(title));
    let address = htm::address(rc_s.server_addr());
    let html = HTML::new()
        .push(head)
        .push(TagDouble::new("body").push(h1).push(address));
    let content = Content::Str(html.to_bytes());
    let ct = rc_s.arc().ents_doc("html");
    map.insert("Content-Length".to_owned(), format!("{}", content.len()));
    map.insert("Content-Type".to_owned(), ct.to_owned());
    let code_name = (*rc_s.arc().cns().get(req.status()).unwrap()).to_owned();
    Response::new(req.protocol(),
                  req.version(),
                  *req.status(),
                  code_name,
                  map,
                  content)
}

fn dir_to_resp(rc_s: Rc<RcStream>, req: &Request, path: &Path, mut map: HashMap<String, String>) -> Response {
    dbstln!("{}@{} dir_to_resp(): {:?}",
            module_path!(),
            rc_s.time().ls(),
            path);
    let content = Content::Str(dir_to_string(rc_s.clone(), req, path));
    let ct = rc_s.arc().ents_doc("html");
    map.insert("Content-Length".to_owned(), format!("{}", content.len()));
    map.insert("Content-Type".to_owned(), ct.to_owned());
    let code_name = (*rc_s.arc().cns().get(req.status()).unwrap()).to_owned();
    Response::new(req.protocol(),
                  req.version(),
                  *req.status(),
                  code_name,
                  map,
                  content)
}

fn dir_to_string(rc_s: Rc<RcStream>, req: &Request, path: &Path) -> Vec<u8> {
    // 如果是route,不提供父目录。
    dbstln!("{:?}", req);
    let path_is_route = rc_s.arc().route_rpset().contains(req.path_rp());
    let title = req.path_vp().to_string();
    let head = htm::head().push(TagDouble::new("title").push(title.as_str()));
    // <span id ="client">127.0.0.1:52622</span></h1>
    let h1 = TagDouble::new("h1")
        .push(title)
        .push(TagDouble::new("span")
                  .add_attr("id", "client")
                  .push(rc_s.client_addr()))
        .push(TagSingle::new("p"));
    let mut table = TagDouble::new("table")
        .add_attr("id", "table")
        .push(TagDouble::new("thead").push(TagDouble::new("tr")
                                               .add_attr("style", "border-bottom: 0.1px solid #000080;")
                                               .push(TagDouble::new("th").push(TagDouble::new("button")
                                                                                   .add_attr("onclick", "sort_by(0)")
                                                                                   .push("Name")))
                                               .push(TagDouble::new("th").push(TagDouble::new("button")
                                                                                   .add_attr("onclick", "sort_by(1)")
                                                                                   .push("Last_modified")))
                                               .push(TagDouble::new("th").push(TagDouble::new("button")
                                                                                   .add_attr("onclick", "sort_by(2)")
                                                                                   .push("Size")))));
    let mut tbody = TagDouble::new("tbody");
    if !path_is_route {
        let path = Path::new(req.path_rp());
        let path_parent = path.parent()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| req.path_vp.to_string());
        dbstln!("path: {}: path_parent: {}", req.path_vp, path_parent);

        let mut tr = TagDouble::new("tr").push(TagDouble::new("td")
                                                   .add_attr("class", "dir")
                                                   .push(TagDouble::new("a")
                                                             .add_attr("href", "../")
                                                             .push("../ Parent Directory")));
        if let Some((s, tm)) = psm_to_stm(&path_parent) {
            tr = tr.push(TagDouble::new("td")
                             .add_attr("data".to_owned(), format!("{}", tm.local().rfc822()))
                             .push(tm.ls()));
            tr = tr.push(TagDouble::new("td")
                             .add_attr("data".to_owned(), format!("{}", s))
                             .push(format!("{}", s)));
        } else {
            tr = tr.push(TagDouble::new("td")
                             .add_attr("data", "--- ---")
                             .push("--- ---"));
            tr = tr.push(TagDouble::new("td").add_attr("data", "--").push("--"));
        }
        tbody = tbody.push(tr);
    }
    let dir_entrys = fs::read_dir(path).unwrap();
    for entry in dir_entrys {
        let entry = entry.unwrap().path();
        let entry_name = entry
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let entry_path = String::new() + req.path_rp() + "/" + &entry_name;
        let path_encoded = match quote(entry_name.clone(), b"") {
            Ok(ok) => ok,
            Err(_) => {
                errstln!("{}@{}_Entry_encoding_error:\n{:?}",
                         module_path!(),
                         rc_s.time().ls(),
                         &path);
                entry_name.clone()
            }
        };

        // 比如当前目录下有断开的链接时会陷入。
        // if !Path::new(&entry_path).exists() {
        //     panic!("{}@{}_path_拼接_error:\n{:?}\n",
        //            module_path!(),
        //            rc_s.time().ls(),
        //            &entry_path);
        // }

        // "/" 区分目录与文件(视觉),并且如果没有它，浏览器不会自动拼路径，这上面坑了好多时间。
        // 仔细对比响应，python3 -m http.server 8000，fuckerfuckf.
        let mut tr = TagDouble::new("tr");
        let (tms_js, tms, ss_js, ss) = {
            if let Some((s, tm)) = psm_to_stm(&entry_path) {
                if Path::new(&entry_path).is_dir() {
                    tr = tr.push(TagDouble::new("td")
                                     .add_attr("class", "dir")
                                     .push(TagDouble::new("a")
                                               .add_attr("href", path_encoded + "/")
                                               .push(entry_name + "/")));
                } else {
                    tr = tr.push(TagDouble::new("td")
                                     .add_attr("class", "file")
                                     .push(TagDouble::new("a")
                                               .add_attr("href", path_encoded)
                                               .push(entry_name)));
                }
                (format!("{}", tm.local().rfc822()), tm.ls().to_owned(), format!("{}", s), format!("{}", s))
            } else {
                if Path::new(&entry_path).is_dir() {
                    tr = tr.push(TagDouble::new("td")
                                     .add_attr("class", "dir")
                                     .push(TagDouble::new("a")
                                               .add_attr("href", path_encoded + "/")
                                               .push(entry_name + "/")));
                } else {
                    tr = tr.push(TagDouble::new("td")
                                     .add_attr("class", "file")
                                     .push(TagDouble::new("a")
                                               .add_attr("href", path_encoded)
                                               .push(entry_name)));
                }
                ("--- --".to_owned(), "--- --".to_owned(), "--".to_owned(), "--".to_owned())
            }
        };
        tr = tr.push(TagDouble::new("td").add_attr("data", tms_js).push(tms));
        tr = tr.push(TagDouble::new("td").add_attr("data", ss_js).push(ss));
        tbody = tbody.push(tr);
    }
    table = table.push(tbody);
    let address = htm::address(rc_s.server_addr());
    let html = HTML::new()
        .push(head)
        .push(TagDouble::new("body")
                  .push(h1)
                  .push(table)
                  .push(TagSingle::new("hr"))
                  .push(address));
    html.to_bytes()
}

fn psm_to_stm(path: &str) -> Option<(u64, Date)> {
    if let Ok((s, m)) = path::pathsm(path) {
        if let Ok(o) = Duration::from_std(m) {
            return Some((s, Date::new(now() - o)));
        }
    }
    None
}
