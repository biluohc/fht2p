use url::percent_encoding::{percent_decode, percent_encode_byte};

// 以后应该自己组结构体，Hyper 的 Url new方法都没, md 一个 path 都改不了，只能反复 decode..
pub fn url_for_parent(path: &str) -> String{
    let req_path_dec = percent_decode(path.as_bytes()).decode_utf8().unwrap();
    let cow_str = req_path_dec.as_ref();
    let req_path_components = cow_str
        .split('/')
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>();
    // if `/` encoding as '%2F', brower will concat origin'path with req_path_parent
    if req_path_components.len() > 1 {
        req_path_components[..req_path_components.len() - 1]
            .iter()
            .flat_map(|c| {
                "/".chars()
                    .chain(c.bytes().flat_map(|cc| percent_encode_byte(cc).chars()))
            })
            .collect::<String>()
    } else {
        "/".to_owned()
    }
}