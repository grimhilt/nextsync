use std::path::Path;
use std::path::PathBuf;
use crate::utils::{head, path};
use  crypto::sha1::Sha1;
use crypto::digest::Digest;
use std::fs::{OpenOptions, self};
use std::io::Write;
use std::io;

pub fn add_tree(path: &Path) {
    dbg!(path.clone());
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let mut hasher = Sha1::new();
    hasher.input_str(path.clone().to_str().unwrap());
    let hash = hasher.result_str();
    let mut line = hash.to_owned();
    line.push_str(" ");
    line.push_str(file_name);
    if path.iter().count() == 1 {
        dbg!(head::add_line(line));
    } else {
        dbg!(add_node(path.parent().unwrap(), &line));
    }
    dbg!(add_file(hash, file_name));
    dbg!(path.iter().count());
}

fn add_node(path: &Path, node: &str) -> io::Result<()> {
    let mut root = match path::nextsync_root() {
        Some(path) => path,
        None => todo!(),
    };
    root.push(".nextsync");
    root.push("objects");
   
    let mut hasher = Sha1::new();
    hasher.input_str(path.clone().to_str().unwrap());
    let hash = hasher.result_str();
    let (dir, rest) = hash.split_at(2);

    root.push(dir);
    if !root.exists() {
        todo!();
    }
    root.push(rest);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(root)?;

    writeln!(file, "{}", node)?;
    Ok(())
}

fn add_file(name: String, content: &str) -> io::Result<()> {
    let mut root = match path::nextsync_root() {
        Some(path) => path,
        None => todo!(),
    };
    root.push(".nextsync");
    root.push("objects");
    let c = name.clone();
    let (dir, rest) = c.split_at(2);

    root.push(dir);
    if !root.exists() {
       fs::create_dir_all(root.clone())?; 
    }
    root.push(rest);
    dbg!(root.clone());
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
