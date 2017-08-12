use super::*;

use std::collections::HashMap as Map;
use std::path::Path;
use std::ffi::OsStr;

const INDEX_HTMLS: [&'static str; 2] = ["index.htm", "index.html"];

lazy_static! { 
    static ref STATIC_FILES: Map<&'static str,&'static [u8]> = {
      let mut sfs =Map::with_capacity(3);
        sfs.insert(FAVICON_ICO_PATH, &FAVICON_ICO[..]);
        sfs.insert(CSS_PATH, CSS.as_bytes());
        sfs.insert(JS_PATH, JS.as_bytes());
        sfs
    };
    static ref ROUTES: StaticMut<Vec<Route>> = {
        StaticMut::new(Vec::new())
    };
}
/// `Url` -> `Path`
#[derive(Debug,Clone,PartialEq)]
pub struct Route {
    img: String,
    rel: String,
    is_sfs: bool,
    // is route in config
    is_route: bool,
    // dir -> dir/index.htm[l]
    is_redirect: bool,
}

impl Route {
    /// Is not static files
    pub fn new<S1: Into<String>, S2: Into<String>>(img: S1, rel: S2) -> Self {
        Route {
            img: img.into(),
            rel: rel.into(),
            is_sfs: false,
            is_route: false,
            is_redirect: false,
        }
    }
    /// Is static files, it's `rel().is_empty()` and `is_dfs()`
    pub fn sfs<S: Into<String>>(img: S) -> Self {
        Route {
            img: img.into(),
            rel: String::with_capacity(0),
            is_sfs: true,
            is_route: false,
            is_redirect: false,
        }
    }
    #[allow(non_snake_case)]
    pub fn Sfs<S: AsRef<str>>(path: S) -> &'static [u8] {
        STATIC_FILES[path.as_ref()]
    }
    pub fn is_sfs(&self) -> &bool {
        &self.is_sfs
    }
    pub fn is_route(&self) -> &bool {
        &self.is_route
    }
    /// header('HTTP/1.1 301 Moved Permanently');
    /// header('Location:index.html');
    /// /dir -> /dir/index.htm[l]
    pub fn is_redirect(&self) -> &bool {
        &self.is_redirect
    }
    pub fn img(&self) -> &str {
        &self.img
    }
    pub fn rel(&self) -> &str {
        &self.rel
    }
    /// Init before use
    pub fn init(map: &Map<String, String>) {
        let mut map: Vec<Route> = map.iter()
            .map(|(k, v)| {
                     let mut route = Route::new(k.clone(), v.clone());
                     route.is_route = true;
                     route
                 })
            .collect();
        for file in STATIC_FILES.keys() {
            map.push(Route::sfs(*file));
        }
        map.sort_by(|a, b| b.img.len().cmp(&a.img.len()));
        let routes = ROUTES.as_mut();
        routes.extend(map);
    }
    /// `None` is 404
    pub fn parse<S: AsRef<str>>(path: S) -> Option<Self> {
        let path = path.as_ref().url_handle_pre();
        if path.is_none() {
            return None;
        }
        let path = path.unwrap();
        let path = path.as_str();

        for route in ROUTES.as_ref() {
            let img = route.img();
            // +1 is for /dir -> /dir/index.htm[l]
            if path.len() + 1 >= img.len() {
                // full match,
                if path == img {
                    // "/" to "/index.htm[l]"
                    if *redirect_root() && img == "/" {
                        if let Some(s) = index_html(route.rel()) {
                            let mut tmp = Self::new(path.to_string(), s);
                            tmp.is_redirect = true;
                            return Some(tmp);
                        }
                    }
                    return Some((*route).clone());
                }

                // index.html[l]
                if img.ends_with('/') && img.starts_with(path) && path.len() + 1 == img.len() {
                    // println!("index_img/_path+1 {:?}-->{:?}",img,path);
                    return index_html(route.rel()).map(|s| {
                                                           let mut tmp = Self::new(path.to_string(), s);
                                                           tmp.is_redirect = true;
                                                           tmp
                                                       });
                }

                // starts with img, >
                // /img/abc -> /rel/abc
                if path.starts_with(img) && !route.is_sfs() {
                    let tail = &path[img.len() - 1..];
                    // println!("{:?} --> {:?}", path, tail);
                    if tail.starts_with('/') {
                        let rel = route.rel().to_string() + &tail[1..];
                        if rel.ends_with('/') && Path::new(&rel).is_dir() || Path::new(&rel).is_file() {
                            return Some(Self::new(path.to_string(), rel));
                        } else {
                            return index_html(rel).map(|s| {
                                                           let mut tmp = Self::new(path.to_string(), s);
                                                           tmp.is_redirect = true;
                                                           tmp
                                                       });
                        }
                    }
                }
            }
        }
        None
    }
}

// header('HTTP/1.1 301 Moved Permanently');
// header('Location:index.html');
// /dir -> /dir/index.htm[l]
fn index_html<S: AsRef<str>>(path: S) -> Option<String> {
    let path = path.as_ref();
    for html in &INDEX_HTMLS {
        if Path::new(path).join(html).as_path().is_file() {
            if path.ends_with('/') {
                return Some(path.to_owned() + html);
            } else {
                return Some(path.to_owned() + "/" + html);
            }
        }
    }
    None
}

/// 除去 `../` 和多余的 `/`,虽然对 `..//home/` 的处理不正确，但也够用了。
trait UrlHandlePre {
    fn url_handle_pre(&self) -> Option<String>;
}
impl UrlHandlePre for String {
    fn url_handle_pre(&self) -> Option<String> {
        self.as_str().url_handle_pre()
    }
}
impl<'a> UrlHandlePre for &'a str {
    fn url_handle_pre(&self) -> Option<String> {
        let mut cpts: Vec<&OsStr> = Vec::new();
        //.components()迭代出的组件会自动消去多余的/,除了/开始的保留首位的/。
        for c in Path::new(self).components() {
            let c = c.as_os_str();
            // println!("{:?}", c);
            if c == OsStr::new("..") {
                cpts.pop();
            } else {
                cpts.push(c);
            }
        }
        let mut raw = String::new();
        // 迭代器处理不为/开始的添加两次/
        if cpts.is_empty() {
            return None;
        }
        let cpts = if cpts[0] == OsStr::new("/") {
            let mut cp = cpts.into_iter();
            raw.push('/');
            cp.next();
            cp
        } else {
            cpts.into_iter()
        };
        raw = cpts.zip(vec![OsStr::new("/")].into_iter().cycle())
            .fold(raw,
                  |acc, (x, y)| acc + x.to_str().unwrap() + y.to_str().unwrap());
        // 去除没有的/
        if !self.ends_with('/') {
            raw.pop();
        }
        //  win 上Path会自动加 "\\" ,然后尽数404
        if cfg!(windows) {
            raw = raw[1..].to_string();
        }
        Some(raw)
    }
}

#[test]
fn test() {
    let mut map: Map<String, String> = Map::new();
    let routes = [("/", "tests/"),
                  ("/route/", "tests/index_for_route/"),
                  ("/abc/", "tests/index_for_route/abc/"),
                  ("/xyz/", "tests/index_for_route/xyz/")];
    routes
        .into_iter()
        .map(|&(img, rel)| map.insert(img.to_owned(), rel.to_owned()))
        .count();
    Route::init(&map);

    // path,is_none(404)
    let paths = vec!["./", "../", "/../", "/../..", "/../../", "/home", "/home/"];
    for path in paths {
        let route = Route::parse(path);
        assert!(route.is_none());
        errln!("{:?}.is_404: {} -> {:?}", path, route.is_none(), route);
    }
    errln!();
    test_sfs("/fht2p.js");
    test_sfs("/fht2p.css");
    test_sfs("/favicon.ico");

    errln!();
    test_sfs_ne("fht2p.js");
    test_sfs_ne("fht2p.css");
    test_sfs_ne("favicon.ico");
    test_sfs_ne("/fht2p.js/");
    test_sfs_ne("/fht2p.css/");
    test_sfs_ne("/favicon.ico/");

    errln!();
    test_route("/", "tests/");
    test_route("/route/", "tests/index_for_route/");
    test_redirect("/route", "tests/index_for_route/index.html");
    test_("/route/lib.rs", "tests/index_for_route/lib.rs");

    errln!();
    test_route("/abc/", "tests/index_for_route/abc/");
    test_redirect("/abc", "tests/index_for_route/abc/index.html");
    test_("/abc/newfile", "tests/index_for_route/abc/newfile");

    errln!();
    test_("/route/abc/", "tests/index_for_route/abc/");
    test_redirect("/route/abc", "tests/index_for_route/abc/index.html");
    test_("/route/abc/newfile", "tests/index_for_route/abc/newfile");

    errln!();
    test_route("/xyz/", "tests/index_for_route/xyz/");
    test_redirect("/xyz", "tests/index_for_route/xyz/index.htm");
    test_("/xyz/main.rs", "tests/index_for_route/xyz/main.rs");

    errln!();
    test_("/route/xyz/", "tests/index_for_route/xyz/");
    test_redirect("/route/xyz", "tests/index_for_route/xyz/index.htm");
    test_("/route/xyz/main.rs", "tests/index_for_route/xyz/main.rs");

    fn test_(img: &str, rel: &str) {
        let route = Route::parse(img);
        errln!("{:?} -> {:?}", img, route);
        let route_ = Route::new(img, rel);
        assert_eq!(route.unwrap(), route_);
    }
    fn test_route(img: &str, rel: &str) {
        let route = Route::parse(img);
        errln!("{:?} -> {:?}", img, route);
        let mut route_ = Route::new(img, rel);
        route_.is_route = true;
        assert_eq!(route.unwrap(), route_);
    }
    fn test_redirect(img: &str, rel: &str) {
        let route = Route::parse(img);
        errln!("{:?} -> {:?}", img, route);
        let mut route_ = Route::new(img, rel);
        route_.is_redirect = true;
        assert_eq!(route.unwrap(), route_);
    }
    fn test_sfs(img: &str) {
        let route = Route::parse(img);
        errln!("{:?} -> {:?}", img, route);
        let route_ = Route::sfs(img);
        assert_eq!(route.unwrap(), route_);
    }
    fn test_sfs_ne(img: &str) {
        let route = Route::parse(img);
        errln!("{:?} -> {:?}", img, route);
        assert!(route.is_none());
    }
}
