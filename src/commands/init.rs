use std::env;
use std::fs::{DirBuilder, File};
use std::path::PathBuf;
use crate::utils::read::read_folder;
use crate::global::global::DIR_PATH;

pub fn init() {
    let d = DIR_PATH.lock().unwrap();

    let mut path = match d.clone() {
        Some(dir) => PathBuf::from(dir),
        None => env::current_dir().unwrap(),
    };

    if let Ok(entries) = read_folder(path.clone()) {
        if entries.len() != 0 {
            eprintln!("fatal: destination path '{}' already exists and is not an empty directory.", path.display());
            std::process::exit(1);
        }   
    } else {
        eprintln!("fatal: cannot open the destination directory");
        std::process::exit(1);
    }

    let builder = DirBuilder::new();
    // todo check if dir empty

    // .nextsync folder
    path.push(".nextsync");
    match builder.create(path.clone()) {
        Ok(()) => (),
        Err(_) => println!("Error: cannot create .nextsync"),
    };

    path.push("objects");
    match builder.create(path.clone()) {
        Ok(()) => (),
        Err(_) => println!("Error: cannot create objects"),
    };
    path.pop();

    path.push("HEAD");
    match File::create(path.clone()) {
        Ok(_) => (),
        Err(_) => println!("Error: cannot create HEAD"),
    }

    path.pop();
    path.push("index");
    match File::create(path.clone()) {
        Ok(_) => (),
        Err(_) => println!("Error: cannot create index"),
    }

    path.pop();
    path.pop();
    path.push(".nextsyncignore");
    
    match File::create(path) {
        Ok(_) => (),
        Err(_) => println!("Error: cannot create .nextsyncignore"),
    }
}
