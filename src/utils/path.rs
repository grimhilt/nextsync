use std::env;
use std::path::{PathBuf, Path};

pub fn nextsync_root() -> Option<PathBuf> {
    let mut path = env::current_dir().unwrap();
    let root = loop {
        path.push(".nextsync");
        if path.exists() {
            path.pop();
            break Some(path);
        }
        path.pop();
        path.pop();
        if path == Path::new("/") {
            break None;
        }
    };
    root
}
