use std::fs::OpenOptions;
use std::path::PathBuf;
use std::io::{self, Write};
use crate::utils::{read, path};

pub fn path() -> PathBuf {
    let mut root = path::nextsync();
    root.push("HEAD");
    root 
}

pub fn add_line(line: String) -> io::Result<()> {
    let root = path();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(root)?;

    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn rm_line(line: &str) -> io::Result<()> {
    let root = path();
    read::rm_line(root, line)?;
    Ok(())
}
