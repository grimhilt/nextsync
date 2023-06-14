use std::fs::OpenOptions;
use std::fs::File;
use std::path::PathBuf;
use crate::utils::read;
use std::io;

pub fn _read_only(mut path: PathBuf) -> File {
    path.push("index");
    OpenOptions::new()
        .read(true)
        .open(path).expect("Cannot open index file")
}

pub fn open(mut path: PathBuf) -> File {
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
