use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use crate::utils::{read, path};
use std::io::{self, Write};

pub fn _read_only(mut path: PathBuf) -> File {
    path.push("HEAD");
    OpenOptions::new()
        .read(true)
        .open(path).expect("Cannot open HEAD file")
}

pub fn _open(mut path: PathBuf) -> File {
    path.push("HEAD");
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(path).expect("Cannot open HEAD file")
}

pub fn _read_line(mut path: PathBuf) -> io::Result<io::Lines<io::BufReader<File>>> {
    path.push("HEAD");
    read::read_lines(path)
}

pub fn add_line(line: String) -> io::Result<()> {
    let mut root = match path::nextsync_root() {
        Some(path) => path,
        None => todo!(),
    };

    root.push(".nextsync");
    root.push("HEAD");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(root)?;

    writeln!(file, "{}", line)?;
    Ok(())
}
