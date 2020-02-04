extern crate fht2plib;

use fht2plib::how;
use std::{
    env,
    fs::{read_to_string, remove_file},
    io,
    panic::catch_unwind,
    path::Path,
    thread,
    time::Duration,
};

// use futures::FutureExt;
use reqwest::{redirect::Policy, Client, StatusCode};
use tokio::{
    process::{Child, Command},
    runtime::{Builder, Runtime},
    time::timeout,
};
use walkdir::WalkDir;

const ARGS: &[&str] = &[
    "run",
    "--release",
    "--",
    "-p",
    "9000",
    "-u",
    "-m",
    "-r",
    "-vvv",
    ".",
    ".gitignore",
];

fn uri(pq: &str) -> String {
    format!("http://127.0.0.1:{}/{}", ARGS[4], pq)
}

async fn start_server() -> Child {
    let mut command = Command::new("cargo");
    command.current_dir(".").args(ARGS);

    let mut child = command.spawn().map_err(|e| eprintln!("exec fht2p failed: {:?}", e)).unwrap();

    if timeout(Duration::from_millis(2380), &mut child).await.is_ok() {
        panic!("start fht2p failed, exited early")
    }

    child
}

const CURDIR: &str = "tests/upload/";

async fn httpt() {
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
    for entry in WalkDir::new("../../src/base")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|p| p.file_type().is_file())
    {
        let path = entry.path();
        let fina = entry.file_name().to_str().expect("entry.file_name().to_str()");
        let upa = format!("{}{}", CURDIR, fina);
        let url = uri(CURDIR);

        println!("{}: {}: {}", path.display(), fina, url);

        post_file(&path, &fina, &upa, &url, &client).await.unwrap();
    }
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
        .args(&["curl", "-vF", fina_arg.as_str(), url])
        .spawn()
        .unwrap()
        .await
        .expect("exec curl failed0");
    if !es.success() {
        panic!("exec curl failed1");
    }

    let get = get_text(upa, client).await.unwrap();
    assert_eq!(get.0.as_u16(), 200);

    let fc = read_to_string(fina).unwrap();
    assert_eq!(get.1, fc);

    let after = get_text(CURDIR, client).await.unwrap();

    assert_eq!(after.0, 200);

    remove_file(fina).ok();

    Ok(())
}

async fn get_text(path: &str, client: &Client) -> Result<(StatusCode, String), how::Error> {
    let url = uri(path);
    let resp = client.get(&url).send().await?;
    let code = resp.status();

    let text = resp.text().await?;
    Ok((code, text))
}

#[test]
fn full_http_test() {
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();

    rt.block_on(async {
        let mut server = start_server().await;

        let rest = thread::spawn(|| {
            catch_unwind(|| {
                let mut rt = Runtime::new().expect("Runtime::new()");
                rt.block_on(httpt());
            })
        })
        .join();

        server.kill().unwrap();

        let _ = rest.unwrap().unwrap();
    })
}
