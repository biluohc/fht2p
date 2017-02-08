const LOG_PATH: &'static str = "/home/viw/Downloads/cache/log";
const LINE_IDX: usize = 3;

// 二分法 模块路径@行数 时间@地址
// 二分法 "2017-0212 12:04:01"@"127.0.0.1:43586" "fht2p::server::methods"@66

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, BufReader};

fn main() {
    println!("二分法叉虫子!");
    let map = log_reader();
    for (k, v) in map.iter() {
        println!("addr->count: {:?}->{}", k, v);
    }
}

fn log_reader() -> BTreeMap<String, usize> {
    let file = File::open(LOG_PATH).unwrap();
    let mut reader = BufReader::new(file);
    let mut lines = String::new();

    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    reader.read_to_string(&mut lines).unwrap();

    for line in lines.lines() {
        if line.starts_with("二分法") {
            let vs: Vec<&str> =
                line.split(" ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if map.contains_key(vs[LINE_IDX]) {
                if let Some(x) = map.get_mut(vs[LINE_IDX]) {
                    *x += 1;
                }
            } else {
                map.insert(vs[LINE_IDX].to_string(), 1);
            }
        }
    }
    map
}