use std::io::{self};
use std::path::Path;
use std::fs::{self};
use crate::utils::path;
use crate::store::head;
use crate::store::object::{parse_path, add_node, create_obj, rm_node};

pub fn add(path: &Path, date: &str) -> io::Result<()> {
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
    create_obj(hash, &content)?;

    Ok(())
}

pub fn rm(path: &Path) -> io::Result<()> {
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
