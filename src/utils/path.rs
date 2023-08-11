use std::env;
use std::fs::canonicalize;
use std::path::{PathBuf, Path, Component};

use crate::global::global::DIR_PATH;

/// Improve the path to try remove and solve .. token.
/// Taken from https://stackoverflow.com/questions/68231306/stdfscanonicalize-for-files-that-dont-exist
///
/// This assumes that `a/b/../c` is `a/c` which might be different from
/// what the OS would have chosen when b is a link. This is OK
/// for broot verb arguments but can't be generally used elsewhere
///
/// This function ensures a given path ending with '/' still
/// ends with '/' after normalization.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let ends_with_slash = path.as_ref()
        .to_str()
        .map_or(false, |s| s.ends_with('/'));
    let mut normalized = PathBuf::new();
    for component in path.as_ref().components() {
        match &component {
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component);
                }
            }
            _ => {
                normalized.push(component);
            }
        }
    }
    if ends_with_slash {
        normalized.push("");
    }
    normalized
}

pub fn normalize_relative(file: &str) -> Result<String, String> {
    let current = match current() {
        Some(p) => p,
        None => {
            return Err("cannot find current location".to_owned());
        }
    };
    
    let p = {
        let tmp_p = current.join(PathBuf::from(file));
        normalize_path(tmp_p)
    };

    let relative_p = match p.strip_prefix(repo_root()) {
        Ok(p) => p,
        Err(_) => return Err("is not in a nextsync repo or doesn't exist".to_owned()),
    };
    Ok(relative_p.to_str().unwrap().to_owned())
}

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

pub fn refs() -> PathBuf {
    let mut path = repo_root();
    path.push(".nextsync");
    path.push("refs");
    path
}

pub fn nextsyncignore() -> Option<PathBuf> {
    let mut path = repo_root();
    path.push(".nextsyncignore");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
