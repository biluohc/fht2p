extern crate fht2plib;
use fht2plib::how;
use std::{
    env,
    fs::{read_to_string, remove_dir, remove_file},
    io,
    panic::catch_unwind,
    path::{Path, PathBuf},
    str, thread,
    time::Duration,
};

use reqwest::{redirect::Policy, Client, StatusCode};
use tokio::{
    process::{Child, Command},
    runtime::{Builder, Runtime},
    time::timeout,
};
use walkdir::WalkDir;

const USER_PASSWORD: &str = "www:basic";
const SERVE_POST: &str = "9000";

const ARGS_GET_POST: &[&str] = &[
    "run",
    "--release",
    "--",
    "-p",
    SERVE_POST,
    "-u",
    "-m",
    "-r",
    "-vvv",
    ".",
    ".gitignore",
];

const ARGS_GET_PROXY_WITH_AUTH: &[&str] = &[
    "run",
    "--release",
    "--",
    "-p",
    SERVE_POST,
    "-vvv",
    "-P",
    "",
    "-a",
    USER_PASSWORD,
    ".",
    ".gitignore",
];

fn uri(pq: &str) -> String {
    format!("http://127.0.0.1:{}/{}", SERVE_POST, pq)
}

async fn start_server(args: &[&str]) -> Child {
    let mut command = Command::new("cargo");
    command.current_dir(".").args(args);

    let mut child = command.spawn().map_err(|e| eprintln!("exec fht2p failed: {:?}", e)).unwrap();

    if timeout(Duration::from_millis(2380), &mut child).await.is_ok() {
        panic!("start fht2p failed, exited early")
    }

    child
}

const CURDIR: &str = "tests/upload/";

async fn httpt_get_post() {
    env::set_current_dir(CURDIR).expect("set_current_dir(tests/upload)");

    let client = Client::builder().redirect(Policy::none()).build().expect("Client::builder()");

    let get = get_text("", &client)
        .await
        .map_err(|e| eprintln!("get / failed: {:?}", e))
        .unwrap();
    assert!(get.0.is_success());
    assert!(get.1.len() > 1);

    let get = get_text("/", &client)
        .await
        .map_err(|e| eprintln!("get // failed: {:?}", e))
        .unwrap();
    assert!(get.0.is_redirection());
    assert!(get.1.len() < 1);

    let get = get_text("tests/dir/index.html/", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.html/ failed: {:?}", e))
        .unwrap();
    assert!(get.0.is_redirection());
    assert!(get.1.len() < 1);

    let get = get_text("tests/dir/index.htm/", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.htm/ failed: {:?}", e))
        .unwrap();
    assert!(get.0.is_redirection());
    assert!(get.1.len() < 1);

    // the path of route is file
    let get = get_text(".gitignore", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.htm/ failed: {:?}", e))
        .unwrap();
    assert!(get.0.is_success());
    assert_eq!(get.1.as_str(), include_str!("../.gitignore"));

    #[cfg(unix)]
    {
        // curl "0.0.0.0:9000/?method=mkdir" -d "mkdir=new%E5%B0%8Ffile"
        // mkdir enabled
        let dir = "new小file";
        let dir_path = format!("../../{}", dir);
        remove_dir(&dir_path).ok();
        let stderr = Command::new("curl")
            .args(&["-vd", "mkdir=new%E5%B0%8Ffile", "-u", USER_PASSWORD, &uri("?method=mkdir")])
            .output()
            .await
            .expect("exec curl failed0")
            .stderr;

        if !str::from_utf8(&stderr)
            .expect("str::from_utf8(stdout)")
            .contains("< HTTP/1.1 200 OK")
        {
            eprintln!("stderr: {}", str::from_utf8(&stderr).unwrap());
            panic!("exec curl failed1");
        }
        assert!(remove_dir(&dir_path).is_ok());

        dir_post_files("../../src/base", &client).await;
        dir_post_files("../dir", &client).await;
    }
}

async fn dir_post_files(dir: &str, client: &Client) {
    println!("dir_post_files: {}", dir);

    let mut files = vec![];

    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|p| p.file_type().is_file())
    {
        let path = entry.path();
        let fina = entry.file_name().to_str().expect("entry.file_name().to_str()");
        let upa = format!("{}{}", CURDIR, fina);
        let url = uri(CURDIR);

        println!("{}: {}: {}", path.display(), fina, url);

        post_file(&path, &fina, &upa, &url, client).await.unwrap();
        files.push((path.to_path_buf(), fina.to_owned(), upa));
    }

    // upload multi files once
    post_files(dir, files, client).await
}

async fn post_file(path: &Path, fina: &str, upa: &str, url: &str, client: &Client) -> io::Result<()> {
    remove_file(fina).ok();

    let get = get_text(upa, client).await.unwrap();
    assert_eq!(get.0.as_u16(), 404);

    let before = get_text(CURDIR, client).await.unwrap();
    assert_eq!(before.0, 200);

    let fina_arg = path.to_str().unwrap();
    let fina_arg = format!("filename=@{}", fina_arg);
    let es = Command::new("curl")
        .args(&["-vF", fina_arg.as_str(), url])
        .spawn()
        .unwrap()
        .await
        .expect("exec curl failed0");
    if !es.success() {
        panic!("exec curl failed1");
    }

    let get = get_text(upa, client).await.unwrap();
    assert_eq!(get.0.as_u16(), 200);

    let upfc = read_to_string(fina).unwrap();
    assert_eq!(get.1, upfc);
    let rawfc = read_to_string(path).unwrap();
    assert_eq!(get.1, rawfc);

    remove_file(fina).ok();

    let after = get_text(CURDIR, client).await.unwrap();
    assert_eq!(after.0, 200);

    Ok(())
}

async fn post_files(dir: &str, files: Vec<(PathBuf, String, String)>, client: &Client) {
    println!("post_files: {}", dir);

    let before = get_text(CURDIR, client).await.unwrap();
    assert_eq!(before.0, 200);

    let mut fina_args = vec![];

    for (path, fina, upa) in &files {
        remove_file(fina).ok();

        let get = get_text(upa, client).await.unwrap();
        assert_eq!(get.0.as_u16(), 404);

        let fina_arg = path.to_str().unwrap();
        let fina_arg = format!("filename=@{}", fina_arg);
        fina_args.push(fina_arg);
    }

    let url = uri(CURDIR);
    let mut args = vec!["-v", url.as_str()];
    fina_args.iter().for_each(|arg| {
        args.push("-F");
        args.push(arg.as_str());
    });

    let es = Command::new("curl")
        .args(&args)
        .spawn()
        .unwrap()
        .await
        .expect("exec curl failed0");
    if !es.success() {
        panic!("exec curl failed1");
    }

    for (path, fina, upa) in &files {
        let get = get_text(upa, client).await.unwrap();
        assert_eq!(get.0.as_u16(), 200);

        let upfc = read_to_string(fina).unwrap();
        assert_eq!(get.1, upfc);

        let rawfc = read_to_string(path).unwrap();
        assert_eq!(get.1, rawfc);

        remove_file(fina).ok();
    }

    let after = get_text(CURDIR, client).await.unwrap();
    assert_eq!(after.0, 200);
}

async fn get_text(path: &str, client: &Client) -> Result<(StatusCode, String), how::Error> {
    let url = uri(path);
    let resp = client.get(&url).send().await?;
    let code = resp.status();

    let text = resp.text().await?;
    Ok((code, text))
}

async fn httpt_get_proxy_with_auth() {
    let client = Client::builder().build().expect("Client::builder()");

    let get = get_text("", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.htm/ failed: {:?}", e))
        .unwrap();
    assert_eq!(get.0, 401);
    assert!(get.1.len() > 1);

    let get = get_text("src/base/", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.htm/ failed: {:?}", e))
        .unwrap();
    assert_eq!(get.0, 401);
    assert!(get.1.len() > 1);

    let get = get_text("src/main.rs", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.htm/ failed: {:?}", e))
        .unwrap();
    assert_eq!(get.0, 401);
    assert!(get.1.len() > 1);

    // the path of route is file
    let get = get_text(".gitignore", &client)
        .await
        .map_err(|e| eprintln!("get /tests/dir/index.htm/ failed: {:?}", e))
        .unwrap();
    assert_eq!(get.0, 401);
    assert!(get.1.len() > 1);

    #[cfg(unix)]
    {
        for upa in &["", "src/base/", "src/main.rs", ".gitignore"] {
            let es = Command::new("curl")
                .args(&["-v", "-u", USER_PASSWORD, &uri(upa)])
                .spawn()
                .unwrap()
                .await
                .expect("exec curl failed0");
            if !es.success() {
                panic!("exec curl failed1");
            }
        }

        // upload disabled
        let fina_arg = format!("filename=@{}", "Cargo.toml");
        let stderr = Command::new("curl")
            .args(&["-vF", fina_arg.as_str(), "-u", USER_PASSWORD, &uri("src/")])
            .output()
            .await
            .expect("exec curl failed0")
            .stderr;

        if !str::from_utf8(&stderr)
            .expect("str::from_utf8(stdout)")
            .contains("< HTTP/1.1 403 Forbidden")
        {
            eprintln!("stderr: {}", str::from_utf8(&stderr).unwrap());
            panic!("exec curl failed1");
        }

        // curl "0.0.0.0:9000/?method=mkdir" -d "mkdir=new%E5%B0%8Ffile"
        // mkdir disabled
        let stderr = Command::new("curl")
            .args(&["-vd", "mkdir=new%E5%B0%8Ffile", "-u", USER_PASSWORD, &uri("?method=mkdir")])
            .output()
            .await
            .expect("exec curl failed0")
            .stderr;

        if !str::from_utf8(&stderr)
            .expect("str::from_utf8(stdout)")
            .contains("< HTTP/1.1 403 Forbidden")
        {
            eprintln!("stderr: {}", str::from_utf8(&stderr).unwrap());
            panic!("exec curl failed1");
        }

        // normal: curl --proxy http://www:yos@127.0.0.1:8000 127.0.0.1:8080/
        let es = Command::new("curl")
            .args(&[
                "-v",
                "-u",
                USER_PASSWORD,
                "-U",
                USER_PASSWORD,
                "--proxy",
                &uri(""),
                &uri(".gitignore"),
            ])
            .spawn()
            .unwrap()
            .await
            .expect("exec curl failed0");
        if !es.success() {
            panic!("exec curl failed1");
        }

        // tunnel: curl --proxy http://www:yos@127.0.0.1:8000 https://tools.ietf.org/favicon.ico
        let es = Command::new("curl")
            .args(&[
                "-v",
                "-u",
                USER_PASSWORD,
                "-U",
                USER_PASSWORD,
                "--proxy",
                &uri(""),
                "--output",
                "-",
                "https://tools.ietf.org/favicon.ico",
            ])
            .spawn()
            .unwrap()
            .await
            .expect("exec curl failed0");
        if !es.success() {
            panic!("exec curl failed1");
        }
    }
}

#[test]
fn full_http_test() {
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();

    rt.block_on(async {
        let mut server = start_server(ARGS_GET_PROXY_WITH_AUTH).await;

        let rest = thread::spawn(|| {
            catch_unwind(|| {
                let mut rt = Runtime::new().expect("Runtime::new()");
                rt.block_on(httpt_get_proxy_with_auth());
            })
        })
        .join();

        server.kill().unwrap();

        let _ = rest.unwrap().unwrap();
    });

    rt.block_on(async {
        let mut server = start_server(ARGS_GET_POST).await;

        let rest = thread::spawn(|| {
            catch_unwind(|| {
                let mut rt = Runtime::new().expect("Runtime::new()");
                rt.block_on(httpt_get_post());
            })
        })
        .join();

        server.kill().unwrap();

        let _ = rest.unwrap().unwrap();
    });
}
