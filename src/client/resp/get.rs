use super::*;
use super::ContentType;
use super::path_info::*;
use super::html::*;

use stderr::Loger;
use urlparse::quote;

use std::fs::{self, File};
use std::path::Path;

include!("path_type.rs");

pub fn get(mut resp: &mut Response, req: &Request) {
    let mut path_type = PathType::Code;
    if let Some(route) = req.route() {
        if !route.is_sfs() {
            let path = Path::new(route.rel());
            if path.is_dir() {
                path_type = PathType::Dir;
            }
            if path.is_file() {
                path_type = PathType::File;
                if *route.is_redirect() {
                    resp.code_set(301_u16);
                }
            }
            if !path.exists() {
                resp.code_set(404_u16);
            } else if path_is_403(path) {
                resp.code_set(403_u16);
                path_type = PathType::Code;
            }
        } else {
            path_type = PathType::Sfs;
        }
    } else {
        resp.code_set(404_u16);
    }
    match resp.code() {
        200 if path_type.is_sfs() => sfs_handle(resp, req),
        200 if path_type.is_file() => (file_handle(resp, req)),
        200 if path_type.is_dir() => dir_handle(resp, req),
        301 if path_type.is_file() => {
            file_handle(resp, req);
            let route = req.route().unwrap();
            if *route.is_redirect() {
                let path = Path::new(route.rel()).file_name().unwrap();
                let img = route.img().to_string() + "/" + &path.to_string_lossy().into_owned();
                resp.header_insert("Location", img);
            }
        }
        _ => {
            assert!(path_type.is_code());
            code_handle(resp, req);
        }
    }
    resp.content_length_insert();
}

pub fn path_is_403<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if path.is_file() {
        File::open(path).is_err()
    } else if path.is_dir() {
        fs::read_dir(path).is_err()
    } else {
        // 套接字，断开的链接，等
        true
    }
}

pub fn sfs_handle(mut resp: &mut Response, req: &Request) {
    let path = req.route().unwrap().img();
    dbstln!("sfs_handle(): {:?}", path);
    let bytes: &'static [u8] = Route::Sfs(path);
    let content_type = ContentType::from_bytes(path, bytes);
    let content = Content::Sf(bytes);
    resp.content_type_insert(content_type);
    resp.content.update(content);
}

pub fn file_handle(mut resp: &mut Response, req: &Request) {
    let path = req.route().unwrap().rel();
    dbstln!("file_handle(): {:?}", path);
    let content = Content::File(File::open(path).unwrap());
    resp.content_type_insert(ContentType::from_path(path));
    resp.content.update(content);
}
pub fn code_handle(mut resp: &mut Response, req: &Request) {
    let path = req.line().path();
    dbstln!("code_handle({}): {:?}", resp.line.code().code(), path);
    let content = Content::Str(code(resp.line.code(), req.client_addr(), req.server_addr()).into_string());
    resp.content_type_insert(ContentType::html());
    resp.content.update(content);
}
pub fn dir_handle(mut resp: &mut Response, req: &Request) {
    let content = Content::Str(dir_to_string(req));
    resp.content_type_insert(ContentType::html());
    resp.content.update(content);
}
#[allow(unknown_lints,or_fun_call)]
fn dir_to_string(req: &Request) -> String {
    let route = req.route().unwrap();
    let path = Path::new(route.rel());
    dbstln!("dir_handle: {:?}", path);
    let title_h1 = route.img();
    let mut dir = Dir::new(title_h1, req.client_addr(), req.server_addr());
    //不是route才提供父目录
    if !route.is_route() {
        let path_parent = path.parent()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap();
        dbstln!("path: {:?}: path_parent: {:?}", path, path_parent);
        dir.push(Entry::new(Class::Dir,
                            "../",
                            "../ Parent Directory",
                            PathInfo::new(path_parent)));
    }
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap().path();
        let mut entry_name = entry.file_name().unwrap().to_string_lossy().into_owned();
        // let entry_path =  path.clone().join(entry_name)
        let mut entry_name_encoded = quote(&entry_name, b"")
            .map_err(|e| {
                         errln!("entry_encoding_error({:?}): {:?} @ {:?}",
                                e,
                                entry_name,
                                &path);
                     })
            .unwrap_or(entry_name.clone());
        if entry.is_dir() {
            entry_name.push('/');
            entry_name_encoded.push('/');
            dir.push(Entry::new(Class::Dir,
                                entry_name_encoded,
                                entry_name,
                                PathInfo::new(entry)));
        } else {
            dir.push(Entry::new(Class::File,
                                entry_name_encoded,
                                entry_name,
                                PathInfo::new(entry)));
        }
    }
    dir.into_maud().into_string()
}
