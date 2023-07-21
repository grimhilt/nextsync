use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions};
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use std::io::{Seek, SeekFrom, Read};
use crate::utils::time::parse_timestamp;
use crate::store::head;
use crate::utils::{read, path};

pub mod tree;
pub mod blob;

pub struct Object {
    path: PathBuf,
    hash: String,
    obj_p: PathBuf,
    ts: Option<i64>
}

impl Object {
    pub fn new(path: &str) -> Object {
        let path = match path.chars().next_back() == "/".chars().next() {
            true => {
                let mut new = path.chars();
                new.next_back();
                new.as_str()
            },
            false => path,
        };
        if path == "" {
            return Object {
                path: PathBuf::from("/"),
                hash: String::from(""),
                obj_p: head::path(),
                ts: None,
            }
        }

        let mut hasher = Sha1::new();
        hasher.input_str(path);
        let hash = hasher.result_str();

        let (dir, res) = hash.split_at(2);

        let mut obj_p = path::objects();
        obj_p.push(dir);
        obj_p.push(res);

        Object {
            path: PathBuf::from(path),
            hash,
            obj_p,
            ts: None,
        } 
    }

    pub fn read(&mut self) -> &mut Object {
        match read::read_lines(&self.obj_p) {
            Ok(mut reader) => {
                if let Some(Ok(line)) = reader.next() {
                    let mut data = line.rsplit(' ');
                    if data.clone().count() >= 2 {
                        self.ts = Some(data.next().unwrap().parse::<i64>().unwrap())
                    }
                }
            },
            Err(err) => {
                eprintln!("error reading object {}: {}", self.obj_p.display(), err);
            },
        };
        self
    }

    pub fn exists(&mut self) -> bool {
       self.obj_p.exists()
    }

    /// return true if the local file is older than the remote one
    pub fn is_older(&mut self, ts: i64) -> bool {
        // todo be aware of the diff of ts format
        ts > self.ts.expect("Should be read before used") / 1000
    }
}

/// Returns (line, hash, name)
///
/// # Examples
/// Input: /foo/bar
/// Result: ("tree hash(/foo/bar) bar", hash(/foo/bar), bar)
pub fn parse_path(path: PathBuf, is_blob: bool) -> (String, String, String) {
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

fn hash_obj(obj: &str) -> (String, String) {
    let mut hasher = Sha1::new();
    hasher.input_str(obj);
    let hash = hasher.result_str();
    let (dir, res) = hash.split_at(2);
    (String::from(dir), String::from(res))
}

fn _object_path(obj: &str) -> PathBuf {
    let mut root = path::objects();
    let (dir, res) = hash_obj(&obj);

    root.push(dir);
    root.push(res);
    root
}

fn rm(hash: &str) -> io::Result<()> {
    let mut root = path::objects();
    let (dir, rest) = hash.split_at(2);
    root.push(dir);
    root.push(rest);
    fs::remove_file(root)?;
    Ok(())
}

fn rm_node(path: &Path, node: &str) -> io::Result<()> {
    let mut root = path::objects();
    let (dir, rest) = hash_obj(path.clone().to_str().unwrap());

    root.push(dir);
    root.push(rest);

    read::rm_line(root, node)?;
    Ok(())
}

fn add_node(path: &Path, node: &str) -> io::Result<()> {
    let mut root = path::objects();
   
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

fn update_dates(mut path: PathBuf, date: &str) -> io::Result<()> {
    let mut obj_p = path::objects();
    
    while path.pop() {
        let (dir, res) = hash_obj(path.to_str().unwrap());
        obj_p.push(dir);
        obj_p.push(res);
        update_date(obj_p.clone(), date.clone())?;
        obj_p.pop();
        obj_p.pop();
    }

    Ok(())
}

pub fn update_date(path: PathBuf, date: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.clone())?;

    let mut buffer = [0; 1];
    file.seek(SeekFrom::Start(0))?;

    // Seek and read until a space is found
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            // Reached the end of the file without finding a space
            break;
        }

        if buffer[0] == b' ' {
            break;
        }
    }

    file.write_all(&date.as_bytes())?;

    Ok(())
}

fn create_obj(name: String, content: &str) -> io::Result<()> {
    let mut root = path::objects();

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

pub fn get_timestamp(path_s: String) -> Option<i64> {
    let mut obj_p = path::objects();

    let (dir, res) = hash_obj(&path_s);
    obj_p.push(dir);
    obj_p.push(res);
    
    match read::read_lines(obj_p) {
        Ok(mut reader) => {
            match reader.next() {
                Some(Ok(line)) => {
                    let mut data = line.rsplit(' ');
                    if data.clone().count() >= 2 {
                        Some(data.next().unwrap().parse::<i64>().unwrap())
                    } else {
                        None
                    }
                },
                _ => None,
            }
        },
        Err(err) => {
            eprintln!("error reading object: {}", err);
            None
        },
    }

}
