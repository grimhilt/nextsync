use std::env;
use std::path::{PathBuf, Path};
use crate::global::global::DIR_PATH;
use std::fs::canonicalize;

pub fn nextsync_root() -> Option<PathBuf> {
    let d = DIR_PATH.lock().unwrap();

    let mut path = match d.clone() {
        Some(dir) => {
            let tmp = PathBuf::from(dir).to_owned();
            if tmp.is_absolute() {
                tmp
            } else {
                let current_dir = env::current_dir().ok()?;
                let abs = current_dir.join(tmp);
                 let canonicalized_path = canonicalize(abs).ok()?;
                 canonicalized_path
            }
            
        },
        None => env::current_dir().ok()?,
    };

    dbg!(path.clone());

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
