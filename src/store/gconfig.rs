use std::env;
use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use crate::utils::read;

fn global_path() -> Option<PathBuf> {
    if let Some(home_dir) = env::var_os("HOME") {
        let mut path = PathBuf::new();
        path.push(home_dir);
        path.push(".nextsync");
        Some(path)
    }
    else
    {
        None
    }
}

pub fn write_token(token: &str) -> io::Result<()> {
    if let Some(mut path_token) = global_path() {
        if !path_token.exists() {
            fs::create_dir_all(path_token.clone())?; 
        }
        path_token.push("token");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path_token)?;

        writeln!(file, "{}", token)?;
    }
    Ok(())
     
}

pub fn read_token() -> Option<String> {
    if let Some(mut path_token) = global_path() {
        if !path_token.exists() {
            return None;
        }
        path_token.push("token");
        if let Ok(lines) = read::read_lines(path_token) {
            for line in lines {
                if let Ok(l) = line {
                    return Some(l);
                }
            } 
        }
    } 
    
    None
}
