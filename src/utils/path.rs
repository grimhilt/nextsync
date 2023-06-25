use std::env;
use std::fs::canonicalize;
use std::path::{PathBuf, Path};
use crate::global::global::DIR_PATH;

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

pub fn repo_root_without_err() -> Option<PathBuf> {
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

pub fn repo_root() -> PathBuf {
    match repo_root_without_err() {
        Some(p) => p,
        None => {
            eprintln!("fatal: not a nextsync repository (or any of the parent directories): .nextsync");
            std::process::exit(1);
        }
    }
}

pub fn nextsync() -> PathBuf {
    let mut path = repo_root();
       path.push(".nextsync");
       path
}

pub fn objects() -> PathBuf {
    let mut path = repo_root();
    path.push(".nextsync");
    path.push("objects");
    path
}

pub fn nextsyncignore() -> Option<PathBuf> {
    let mut path = repo_root();
    path.push(".nextsyncignore");
    if path.exists() {
        return Some(path);
    } else {
        return None;
    }
    None
}
