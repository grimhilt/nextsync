use std::fs::File;
use std::io::{self};
use std::path::Path;
use crate::utils::{read, path};
use crate::store::head;
use crate::store::object::{self, parse_path, hash_obj, add_node, create_obj};

pub fn add(path: &Path, date: &str) -> io::Result<()> {
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
    create_obj(hash, &content)?;

    Ok(())
}

pub fn rm(path: &Path) -> io::Result<()> {
    let (_, lines) = read(path.to_path_buf().to_str().unwrap().to_string()).unwrap();
    for line in lines {
        let (ftype, hash, _) = parse_line(line.unwrap());
        if ftype == String::from("blob") {
            object::rm(&hash);
        } else {
            rm_hash(hash);
        }
    }
    Ok(())
}

fn rm_hash(hash: String) {
    let mut obj_p = path::objects();
    let (dir, res) = hash.split_at(2);
    obj_p.push(dir);
    obj_p.push(res);

    match read::read_lines(obj_p) {
        Ok(mut reader) => {
            reader.next();
            for line in reader {
                let (ftype, hash, _) = parse_line(line.unwrap());
                if ftype == String::from("blob") {
                    object::rm(&hash);
                } else {
                    rm_hash(hash);
                }
            }
        },
        Err(err) => {
            eprintln!("error reading tree: {}", err);
        },
    }
}

pub fn read(tree: String) -> Option<(String, io::Lines<io::BufReader<File>>)> {
    let mut obj_p = path::objects();

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
