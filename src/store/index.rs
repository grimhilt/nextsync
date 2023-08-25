use std::io;
use std::path::PathBuf;
use std::fs::File;
use std::fs::OpenOptions;
use crate::utils::{read, path};

pub fn path() -> PathBuf {
    let mut path = path::nextsync();
    path.push("index");
    path
}

pub fn open() -> File {
    let mut path = path();
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(path).expect("Cannot open index file")
}

pub fn read_line() -> io::Result<io::Lines<io::BufReader<File>>> {
    let mut path = path::nextsync();
    path.push("index");
    read::read_lines(path)
}

pub fn rm_line(line: &str) -> io::Result<()> {
    let mut root = path::nextsync();
    root.push("index");
    read::rm_line(root, line)?;
    Ok(())
}
