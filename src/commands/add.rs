use clap::Values;
use crate::utils;

pub fn add(files: Values<'_>) {
    let root = match utils::path::nextsync_root() {
        Some(path) => path,
        None => {
            eprintln!("fatal: not a nextsync repository (or any of the parent directories): .nextsync");
            std::process::exit(1);
        } 
    };

    dbg!(root.clone());
    let file_vec: Vec<&str> = files.collect();
    for file in file_vec {
        println!("{}", file);
    } 
}
