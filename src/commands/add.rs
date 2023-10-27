use std::io::Write;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use clap::Values;
use crate::store::index;
use crate::store::{self, object::Object};
use crate::utils;
use crate::utils::nextsyncignore::{self, ignore_file};
use crate::utils::path::{normalize_relative, repo_root, path_buf_to_string};

use super::status::get_all_objs;

pub struct AddArgs<'a> {
    pub files: Option<Values<'a>>,
    pub force: bool,
    pub all: bool,
}

// todo match deleted files
// todo match weird reg expression
pub fn add(args: AddArgs) {
    // write all modification in the index
    if args.all {
        write_all();
        return;
    }

    let mut index_file = store::index::open();
    let mut added_files: Vec<String> = vec![];

    let rules = match nextsyncignore::read_lines() {
        Ok(r) => r,
        Err(_) => vec![],
    };

    let mut ignored_f = vec![];
    let file_vec: Vec<&str> = args.files.unwrap().collect();
    for file in file_vec {
        
        let f = match normalize_relative(file) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("err: {} {}", file, err);
                continue;
            }
        };

        if !args.force && ignore_file(&f, rules.clone(), &mut ignored_f) {
            continue;
        }

        let path = repo_root().join(Path::new(&f));

        match path.exists() {
            true => {
                if path.is_dir() {
                    add_folder_content(path.to_path_buf(), &mut added_files);
                }
                added_files.push(f);
            },
            false => {
                if Object::new(path.to_str().unwrap()).exists() {
                    added_files.push(String::from(f));
                } else {
                    // todo applies regex
                    eprintln!("err: {} is not something you can add.", path.to_str().unwrap());
                }
            }
        }
    } 

    if ignored_f.len() > 0 {
        // todo multiple nextsyncignore
        println!("The following paths are ignored by your .nextsyncignore file:");
        for file in ignored_f {
            println!("{}", file);
        }
    }

    // save all added_files in index
    // todo avoid duplication
    for file in added_files {
        match writeln!(index_file, "{}", file) {
            Ok(()) => (),
            Err(err) => eprintln!("{}", err),
        }
    }
    drop(index_file);
}

fn add_folder_content(path: PathBuf, added_files: &mut Vec<String>) {
    // todo check for changes
    let mut folders: Vec<PathBuf> = vec![];
    folders.push(path);

    while let Some(folder) = folders.pop() {
        if let Ok(entries) = utils::read::read_folder(folder.clone()) {
            for entry in entries {
                let path_entry = PathBuf::from(entry);
                if  path_entry.is_dir() {
                    folders.push(path_entry.clone());
                }
                added_files.push(String::from(path_entry.to_str().unwrap()));
            }
        } 
    }
}

fn write_all() {
    let objs = get_all_objs();
    let mut index_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(index::path()).expect("Cannot open index file");
    for obj in objs {
        let _ = writeln!(index_file, "{}", path_buf_to_string(obj.path.clone()));
    }
}
