use std::fs::{DirBuilder, File};
use std::env;

pub fn init() {
    let builder = DirBuilder::new();
    let mut path = env::current_dir().unwrap();

    // .nextsync folder
    path.push(".nextsync");
    match builder.create(path.clone()) {
        Ok(()) => println!("Directory successfuly created"),
        Err(_) => println!("Error: cannot create directory"),
    }

    path.push("HEAD");
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
