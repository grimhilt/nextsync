use clap::Values;
use crate::utils;
use std::path::Path;
use std::io::Write;

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
    let mut index_file = utils::index::open(index_path);
    // todo avoid duplicate
    // ./folder ./folder/file

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