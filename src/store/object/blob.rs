use std::io;
use std::path::PathBuf;
use std::fs;
use crate::utils::path;
use crate::store::head;
use crate::store::object::{update_dates, parse_path, add_node, create_obj, rm_node};

pub fn add(path: PathBuf, date: &str, up_parent: bool) -> io::Result<()> {
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

    // update date for all parent
    if up_parent {
        update_dates(path, date)?;
    }

    Ok(())
}

pub fn rm(path: PathBuf) -> io::Result<()> {
    let (line, hash, _) = parse_path(path.clone(), true);

    // remove blob reference to parent
    if path.iter().count() == 1 {
        head::rm_line(&line)?;
    } else {
        rm_node(path.parent().unwrap(), &line)?;
    }

    // remove blob object
    let mut root = path::objects();

    let c = hash.clone();
    let (dir, rest) = c.split_at(2);
    root.push(dir);
    root.push(rest);
    fs::remove_file(root)?;

    Ok(())
}
