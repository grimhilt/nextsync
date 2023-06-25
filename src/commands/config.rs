use std::fs::OpenOptions;
use std::io::{self, Write};
use crate::utils::{path, read};

pub fn set(var: &str, val: &str) -> io::Result<()> {
    let mut root = path::nextsync();
    root.push("config");
     
    // todo check if exist
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(root)?;

    let mut line = var.to_owned();
    line.push_str(" ");
    line.push_str(val);
    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn get(var: &str) -> Option<String> {
    let mut root = path::nextsync();
    root.push("config");
    
    if let Ok(lines) = read::read_lines(root) {
        for line in lines {
            if let Ok(l) = line {
                if l.starts_with(var.clone()) {
                    let (_, val) = l.split_once(" ").unwrap();
                    return Some(val.to_owned());
                }
            }
        } 
    }
    None
}
