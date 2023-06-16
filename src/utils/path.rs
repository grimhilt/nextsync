use std::env;
use std::path::{PathBuf, Path};
use crate::global::global::DIR_PATH;
use std::fs::canonicalize;

pub fn current() -> Option<PathBuf> {
    let d = DIR_PATH.lock().unwrap();

    match d.clone() {
        Some(dir) => {
            let tmp = PathBuf::from(dir).to_owned();
            if tmp.is_absolute() {
               Some(tmp) 
            } else {
                let current_dir = env::current_dir().ok()?;
                let abs = current_dir.join(tmp);
                let canonicalized_path = canonicalize(abs).ok()?;
                Some(canonicalized_path) 
            }
            
        },
        None => Some(env::current_dir().ok()?),
    }
}

pub fn nextsync_root() -> Option<PathBuf> {
    let mut path = current()?;

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


pub fn nextsync() -> Option<PathBuf> {
    if let Some(mut path) = nextsync_root() {
       path.push(".nextsync");
       return Some(path);
    }
    None
}

pub fn objects() -> Option<PathBuf> {
    if let Some(mut path) = nextsync_root() {
       path.push(".nextsync");
       path.push("objects");
       return Some(path);
    }
    None
}

pub fn nextsyncignore() -> Option<PathBuf> {
    if let Some(mut path) = nextsync_root() {
       path.push(".nextsyncignore");
       if path.exists() {
           return Some(path);
       } else {
           return None;
       }
    }
    None
}
