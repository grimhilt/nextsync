use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use crate::utils::{read, path};

pub fn add_line(line: String) -> io::Result<()> {
    let mut root = path::nextsync();
    root.push("HEAD");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(root)?;

    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn rm_line(line: &str) -> io::Result<()> {
    let mut root = path::nextsync();
    root.push("HEAD");
    read::rm_line(root, line)?;
    Ok(())
}
