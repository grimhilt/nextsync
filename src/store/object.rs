use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, self};
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use crate::utils::{read, path};
use crate::store::head;

/// Returns (line, hash, name)
///
/// # Examples
/// Input: /foo/bar
/// Result: ("tree hash(/foo/bar) bar", hash(/foo/bar), bar)
fn parse_path(path: &Path, is_blob: bool) -> (String, String, String) {
    let file_name = path.file_name().unwrap().to_str().unwrap();

    let mut hasher = Sha1::new();
    hasher.input_str(path.clone().to_str().unwrap());
    let hash = hasher.result_str();

    let mut line = String::from(if is_blob { "blob" } else { "tree" });
    line.push_str(" ");
    line.push_str(&hash);
    line.push_str(" ");
    line.push_str(file_name);
    (line, hash, String::from(file_name))
}

pub fn parse_line(line: String) -> (String, String, String) {
    let mut split = line.rsplit(' ');
    if split.clone().count() != 3 {
        eprintln!("fatal: invalid object(s)");
        std::process::exit(1);
    }

    let name = split.next().unwrap();
    let hash = split.next().unwrap();
    let ftype = split.next().unwrap();
    (String::from(ftype), String::from(hash), String::from(name))
}

pub fn add_tree(path: &Path, date: &str) -> io::Result<()> {
    let (line, hash, name) = parse_path(path.clone(), false);

    // add tree reference to parent
    if path.iter().count() == 1 {
        head::add_line(line)?;
    } else {
        add_node(path.parent().unwrap(), &line)?;
    }

    // create tree object
    let mut content = name;
    content.push_str(" ");
    content.push_str(date);
    create_object(hash, &content)?;

    Ok(())
}

pub fn rm_blob(path: &Path) -> io::Result<()> {
    let (line, hash, _) = parse_path(path.clone(), true);

    // remove blob reference to parent
    if path.iter().count() == 1 {
        head::rm_line(&line)?;
    } else {
        rm_node(path.parent().unwrap(), &line)?;
    }

    // remove blob object
    let mut root = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };

    let c = hash.clone();
    let (dir, rest) = c.split_at(2);
    root.push(dir);
    root.push(rest);
    fs::remove_file(root)?;

    Ok(())

}

pub fn add_blob(path: &Path, date: &str) -> io::Result<()> {
    let (line, hash, name) = parse_path(path.clone(), true);
    // add blob reference to parent
    if path.iter().count() == 1 {
        head::add_line(line)?;
    } else {
        add_node(path.parent().unwrap(), &line)?;
    }

    let mut content = name.clone().to_owned();
    content.push_str(" ");
    content.push_str(date);

    // create blob object
    create_object(hash, &content)?;

    Ok(())
}

fn hash_obj(obj: &str) -> (String, String) {
    let mut hasher = Sha1::new();
    hasher.input_str(obj);
    let hash = hasher.result_str();
    let (dir, res) = hash.split_at(2);
    (String::from(dir), String::from(res))
}

fn _object_path(obj: &str) -> PathBuf {
    let mut root = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };
    
    let (dir, res) = hash_obj(&obj);
    root.push(dir);
    root.push(res);
    root
}

pub fn read_tree(tree: String) -> Option<(String, io::Lines<io::BufReader<File>>)> {
    let mut obj_p = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };

    let (dir, res) = hash_obj(&tree);
    obj_p.push(dir);
    obj_p.push(res);
    
    match read::read_lines(obj_p) {
        Ok(mut reader) => {
            let name = match reader.next() {
                Some(Ok(line)) => line,
                _ => String::from(""),
            };
            Some((name, reader))
        },
        Err(err) => {
            eprintln!("error reading tree: {}", err);
            None
        },
    }
}

fn rm_node(path: &Path, node: &str) -> io::Result<()> {
    let mut root = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };
   
    let (dir, rest) = hash_obj(path.clone().to_str().unwrap());

    root.push(dir);
    root.push(rest);

    read::rm_line(root, node)?;
    Ok(())
}

fn add_node(path: &Path, node: &str) -> io::Result<()> {
    let mut root = match path::objects() {
        Some(path) => path,
        None => todo!(),
    };
   
    let (dir, rest) = hash_obj(path.clone().to_str().unwrap());

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

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root)?;
    writeln!(file, "{}", content)?;
    Ok(())
}
