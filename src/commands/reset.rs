use std::fs::File;
use crate::utils;

pub fn reset() {
    let mut root = match utils::path::nextsync_root() {
        Some(path) => path,
        None => {
            eprintln!("fatal: not a nextsync repository (or any of the parent directories): .nextsync");
            std::process::exit(1);
        } 
    };
    root.push(".nextsync");
    root.push("index");
    if File::create(root).is_err() {
        eprintln!("fatal: failed to reset");
    }
}
