use std::fs::{DirBuilder, File};
use std::path::PathBuf;
use std::env;
use crate::global::global::DIR_PATH;

pub fn init() {
    let d = DIR_PATH.lock().unwrap();

    let mut path = match d.clone() {
        Some(dir) => PathBuf::from(dir),
        None => env::current_dir().unwrap(),
    };
    let builder = DirBuilder::new();
    // todo check if dir empty

    // .nextsync folder
    path.push(".nextsync");
    match builder.create(path.clone()) {
        Ok(()) => println!("Directory successfuly created"),
        Err(_) => println!("Error: cannot create directory"),
    };

    path.push("objects");
    match builder.create(path.clone()) {
        Ok(()) => println!("Directory successfuly created"),
        Err(_) => println!("Error: cannot create directory"),
    };
    path.pop();

    path.push("HEAD");
    match File::create(path.clone()) {
        Ok(_) => println!("File successfuly created"),
        Err(_) => println!("Error: cannot create .nextsyncignore"),
    }

    path.pop();
    path.push("index");
    match File::create(path.clone()) {
        Ok(_) => println!("File successfuly created"),
        Err(_) => println!("Error: cannot create .nextsyncignore"),
    }

    path.pop();
    path.pop();
    path.push(".nextsyncignore");
    
    match File::create(path) {
        Ok(_) => println!("File successfuly created"),
        Err(_) => println!("Error: cannot create .nextsyncignore"),
    }
}
