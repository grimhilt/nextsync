use std::fs::File;
use crate::utils;

pub fn reset() {
    let mut root = utils::path::nextsync();
    root.push("index");
    if File::create(root).is_err() {
        eprintln!("fatal: failed to reset");
    }
}
