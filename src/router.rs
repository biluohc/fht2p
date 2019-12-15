use args::Route;

use std::path::PathBuf;

pub fn find(routes: &Vec<Route>, req_path: &str) -> Option<(String, PathBuf, bool)> {
    let components_raw = req_path.split('/').filter(|c| !c.is_empty() && c != &".");

    // handle ..
    let reqpath_components = components_raw
        .fold(Ok(vec![]), |cs, c| {
            cs.and_then(move |mut cs| match (cs.len() > 0, c == "..") {
                (_, false) => {
                    cs.push(c);
                    Ok(cs)
                }
                (true, true) => {
                    cs.pop();
                    Ok(cs)
                }
                (false, true) => Err(cs),
            })
        })
        .unwrap_or_else(|e| e);
    debug!("{} -> {:?}", req_path, reqpath_components);
    
    // remove routes that the count of components bg than reqpath_components
    let mut routes = (0..routes.len())
        .into_iter()
        .fold(Vec::with_capacity(routes.len()), |mut rs, idx| {
            if routes[idx].url_components.len() <= reqpath_components.len() {
                rs.push(&routes[idx]);
            }
            rs
        });

    // if not match, remove from routes
    #[allow(unknown_lints, needless_range_loop)]
    for idx in 0..reqpath_components.len() {
        let rpc = reqpath_components[idx];
        let mut cs_idx = routes.len();
        while cs_idx > 0 {
            let cs_idx_tmp = cs_idx - 1;
            if routes[cs_idx_tmp].url_components.len() > idx && routes[cs_idx_tmp].url_components[idx] != rpc {
                routes.remove(cs_idx_tmp);
            }
            cs_idx -= 1;
        }
    }

    // select one from maxlen to minilen
    for r in routes.into_iter().rev() {
        let mut extract_path = reqpath_components[r.url_components.len()..]
            .iter()
            .fold(PathBuf::from(&r.path), |mut p, &c| {
                p.push(c);
                p
            });
        if extract_path.exists() {
            let mut path = reqpath_components
                .into_iter()
                .fold(String::with_capacity(req_path.len()), |mut p, c| {
                    p.push('/');
                    p.push_str(c);
                    p
                });
            if req_path.ends_with('/') {
                path.push('/');
            }
            return Some((path, extract_path, r.redirect_html));
        }
    }
    None
}
