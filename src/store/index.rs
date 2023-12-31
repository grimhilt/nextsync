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
    let path = path();
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

pub fn alread_added(file: String) -> bool {
    if let Ok(lines) = read_line() {
        for line in lines {
            if let Ok(l) = line {
                if l == file {
                    return true;
                }
            }
        }
    }
    return false;
}
