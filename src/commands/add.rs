use clap::Values;
use crate::utils::{self};
use crate::utils::nextsyncignore::{self, ignore_file};
use crate::store;
use std::path::{Path, PathBuf};
use std::io::Write;

pub struct AddArgs<'a> {
    pub files: Values<'a>,
    pub force: bool,
}

pub fn add(args: AddArgs) {
    let mut index_file = store::index::open();
    let mut added_files: Vec<String> = vec![];
    let rules = match nextsyncignore::read_lines() {
        Ok(r) => r,
        Err(_) => vec![],
    };
    let mut ignored_f = vec![];
    let file_vec: Vec<&str> = args.files.collect();
    for file in file_vec {
        if !args.force && ignore_file(&file.to_string(), rules.clone(), &mut ignored_f) {
            continue;
        }
        let path = Path::new(file);
        match path.exists() {
            true => {
                if path.is_dir() {

                    added_files.push(String::from(path.to_str().unwrap()));
                    add_folder_content(path.to_path_buf(), &mut added_files);
                } else {
                    added_files.push(String::from(path.to_str().unwrap()));
                }
            },
            false => {
                // todo deleted file/folder verif if exists
                added_files.push(String::from(path.to_str().unwrap()));
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
