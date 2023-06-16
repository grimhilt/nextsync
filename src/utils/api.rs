use std::path::{PathBuf, Path};
use std::env;

pub fn get_local_path(p: String, local_p: PathBuf, username: &str, dist_p: &str) -> PathBuf {
    let mut final_p = Path::new(p.as_str());
    final_p = final_p.strip_prefix("/remote.php/dav/files/").unwrap();
    final_p = final_p.strip_prefix(username.clone()).unwrap();
    let dist_p = Path::new(dist_p).strip_prefix("/");
    final_p = final_p.strip_prefix(dist_p.unwrap()).unwrap();
    local_p.clone().join(final_p.clone())
}

pub fn get_local_path_t(p: &str) -> String {
    dbg!(p.clone());
    let username = env::var("USERNAME").unwrap();
    let root = env::var("ROOT").unwrap();
    let mut final_p = p;
    final_p = final_p.strip_prefix("/remote.php/dav/files/").unwrap();
    final_p = final_p.strip_prefix(&username).unwrap();
    final_p = final_p.strip_prefix("/").unwrap();
    final_p = final_p.strip_prefix(&root).unwrap();
    final_p.to_string()
}
