use std::path::Path;
use crate::utils::{head, path};
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use std::fs::{OpenOptions, self};
use std::io::{self, Write};

/// Returns (line, hash, name)
///
/// # Examples
/// Input: /foo/bar
/// Result: ("tree hash(/foo/bar) bar", hash(/foo/bar), bar)
fn parse_path(path: &Path) -> (String, String, String) {
    let file_name = path.file_name().unwrap().to_str().unwrap();

    let mut hasher = Sha1::new();
    hasher.input_str(path.clone().to_str().unwrap());
    let hash = hasher.result_str();

    let mut line = String::from("tree");
    line.push_str(" ");
    line.push_str(&hash);
    line.push_str(" ");
    line.push_str(file_name);
    (line, hash, String::from(file_name))
}

pub fn parse_line(line: String) -> (String, String, String) {
    let mut split = line.rsplit(' ');
    if split.clone().count() != 3 {
        dbg!(split.count());
        eprintln!("fatal: invalid object(s)");
        std::process::exit(1);
    }
    let ftype = split.next().unwrap();
    let hash = split.next().unwrap();
    let name = split.next().unwrap();
    (String::from(ftype), String::from(hash), String::from(name))
}

pub fn add_tree(path: &Path) -> io::Result<()> {
    let (line, hash, name) = parse_path(path.clone());

    // add tree reference to parent
    if path.iter().count() == 1 {
        head::add_line(line)?;
    } else {
        add_node(path.parent().unwrap(), &line)?;
    }

    // create tree object
    create_object(hash, &name)?;

    Ok(())
}

fn add_node(path: &Path, node: &str) -> io::Result<()> {
    let mut root = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };
   
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

fn create_object(name: String, content: &str) -> io::Result<()> {
    let mut root = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };

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
