use super::*;
use super::htm::*;
use std::io::BufReader;
use std::fs::{self, File};

pub fn get(rc_s: Rc<RcStream>, mut stream: &mut TcpStream, mut req: Request) {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("Server".to_owned(), format!("{}/{}", NAME, VERSION));
    if rc_s.keep_alive() {
        map.insert("Connection".to_owned(), "keep-alive".to_owned());
    } else {
        map.insert("Connection".to_owned(), "close".to_owned());
    }

    let path = req.path_rp().to_string();
    let path = Path::new(&path);
    // 路径是断开的链接或不是一般文件/目录(套接字等)或权限不足，文件系统IO错误？
    if path.exists() && path_is_403(&path) {
        req.status_set(403);
    }
    // 路径不存在且不是static_files.
    if !path.exists() && !rc_s.arc().sfs().contains_key(req.path_rp().as_str()) {
        req.status_set(404);
    }

    let status_code = req.status;
    // 打印GET方法的日志
    // 127.0.0.1--[2017-0129 21:11:59] 200 "GET /cargo/ HTTP/1.1" @ " "
    println!(r#"{}**[{}] {} "{} {} {}/{}" -> "{}""#,
             rc_s.client_addr(),
             rc_s.time().as_str(),
             status_code,
             req.method(),
             req.path_raw(),
             req.protocol(),
             req.version(),
             req.path_rp);
    let resp = match path.exists() {
        // 正常的文件目录
        true if (status_code == 200 || status_code == 304) && path.is_dir() => dir_to_resp(rc_s.clone(), &req, path, map),
        true if (status_code == 200 || status_code == 304) && path.is_file() => file_to_resp(rc_s.clone(), &req, path, map),
        // 403的文件目录
        true if status_code == 403 => other_status_code_to_resp(rc_s.clone(), &req, map),
        // 静态文件
        false if status_code == 200 || status_code == 304 => sfs_to_resp(rc_s.clone(), &req, path, map),
        // 404的url，文件目录，（注意如果url直接当路径访问，有可能存在.)
        _ if status_code == 404 => other_status_code_to_resp(rc_s.clone(), &req, map),
        // 其它的以后再处理。
        _ => unreachable!(),
    };
    resp.write_response(&mut stream);
    dbstln!();
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
            rc_s.time().as_str(),
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
        .unwrap_or(OsStr::new("*"))
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
            rc_s.time().as_str(),
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

fn other_status_code_to_resp(rc_s: Rc<RcStream>, req: &Request, mut map: HashMap<String, String>) -> Response {
    dbstln!("{}@{} other_status_code_to_resp(): {:?}",
            module_path!(),
            rc_s.time().as_str(),
            req.status);
    let content = Content::Str(other_status_code_html(rc_s.clone(), req));
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
            rc_s.time().as_str(),
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

fn dir_to_string(rc_s: Rc<RcStream>, req: &Request, path: &Path) -> String {
    // 如果是route,不提供父目录。
    dbstln!("{:?}", req);
    let path_is_route = rc_s.arc().route_rpset().contains(req.path_rp());
    let title = req.path_vp().to_string();
    let h1 = title.clone();
    let mut ul = htm::Ul::new();
    if !path_is_route {
        let path = Path::new(req.path_rp());
        let path_parent = path.parent()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or(req.path_vp.to_string());
        dbstln!("path: {}: path_parent: {}", req.path_vp, path_parent);

        let (date, size) = path::fms(&path_parent);
        ul.push(htm::Li::new("../".to_owned(),
                             "../ Parent Directory".to_string(),
                             date,
                             size));
    }
    let dir_entrys = fs::read_dir(path).unwrap();
    for entry in dir_entrys {
        let entry = entry.unwrap().path();
        let entry_name = entry.file_name().unwrap().to_string_lossy().into_owned();
        let entry_path = String::new() + req.path_rp() + "/" + &entry_name;
        let (date, size) = path::fms(&entry_path);

        let path_encoded = match quote(entry_name.clone(), b"") {
            Ok(ok) => ok,
            Err(_) => {
                errstln!("{}@{}_Entry_encoding_error:\n{:?}",
                         module_path!(),
                         rc_s.time().as_str(),
                         &path);
                entry_name.clone()
            }
        };

        if !Path::new(&entry_path).exists() {
            panic!("{}@{}_path_拼接_error:\n{:?}\n",
                   module_path!(),
                   rc_s.time().as_str(),
                   &entry_path);
        }
        if Path::new(&entry_path).is_dir() {
            // "/" 区分目录与文件(视觉),并且如果没有它，浏览器不会自动拼路径，这上面坑了好多时间。
            // 仔细对比响应，python3 -m http.server 8000，fuckerfuckf.
            ul.push(htm::Li::new(path_encoded + "/", entry_name + "/", date, size))
        } else {
            ul.push(htm::Li::new(path_encoded, entry_name, date, size))
        }
    }
    let addr = htm::Address::new(rc_s.server_addr().to_string());
    let html = htm::Html::new(title,
                              htm::H1::new(h1, rc_s.client_addr().to_string()),
                              Some(ul),
                              addr);
    format!("{}", html)
}
