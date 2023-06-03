use clap::Values;
use std::fs::OpenOptions;
use crate::utils;
use std::path::Path;
use std::io::Write

pub fn add(files: Values<'_>) {
    let root = match utils::path::nextsync_root() {
        Some(path) => path,
        None => {
            eprintln!("fatal: not a nextsync repository (or any of the parent directories): .nextsync");
            std::process::exit(1);
        } 
    };

    let mut index_path = root.clone();
    index_path.push(".nextsync");
    index_path.push("index");
    let mut index_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(index_path).expect("Cannot open index file");

    let file_vec: Vec<&str> = files.collect();
    for file in file_vec {
        let path = Path::new(file);
        println!("{}", file);
        match path.try_exists() {
            Ok(true) => {
                match writeln!(index_file, "{}", path.display()) {
                    Ok(()) => (),
                    Err(err) => eprintln!("{}", err),
                }
            },
            Ok(false) => {
                // todo can be regex
            },
            Err(err) => { 
                eprintln!("Error: {}", err);
            }
        }
    } 
}
