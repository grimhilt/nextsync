use std::fs::OpenOptions;
use std::fs::File;
use std::path::PathBuf;
use crate::utils::{read, path};
use std::io;

pub fn _read_only(mut path: PathBuf) -> File {
    path.push("index");
    OpenOptions::new()
        .read(true)
        .open(path).expect("Cannot open index file")
}

pub fn open() -> File {
    let mut path = match path::nextsync() {
        Some(p) => p,
        None => todo!(),
    };

    path.push("index");
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(path).expect("Cannot open index file")
}

pub fn read_line(mut path: PathBuf) -> io::Result<io::Lines<io::BufReader<File>>> {
    path.push("index");
    read::read_lines(path)
}

pub fn rm_line(line: &str) -> io::Result<()> {
    let mut root = match path::nextsync() {
        Some(path) => path,
        None => todo!(),
    };

    root.push("index");
    read::rm_line(root, line)?;
    Ok(())
}
