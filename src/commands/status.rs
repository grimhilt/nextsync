use crate::utils;
use std::fs::File;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::collections::HashSet;
use colored::Colorize;
use std::path::PathBuf;
use std::io;

#[derive(PartialEq)]
enum RemoveSide {
    Left,
    Both,
    Right,
}

// todo: relative path, filename, get modified
pub fn status() {
    let mut hashes = HashSet::new();
    let mut objects: Vec<String> = vec![];
    let mut staged_objects: Vec<String> = vec![];

    let root = match utils::path::nextsync_root() {
        Some(path) => path,
        None => {
            eprintln!("fatal: not a nextsync repository (or any of the parent directories): .nextsync");
            std::process::exit(1);
        } 
    };

    let mut next_sync_path = root.clone();
    next_sync_path.push(".nextsync");

    if let Ok(lines) = read_head(next_sync_path.clone()) {
        for line in lines {
            if let Ok(ip) = line {
                dbg!(ip.clone().len());
                if ip.clone().len() > 5 {
                    hashes.insert(String::from(ip));
                }
            }
        }
    }

    if let Ok(entries) = utils::read::read_folder(root.clone()) {
        for entry in entries {
            if !is_nextsync_config(entry.clone()) {
                let object_path = entry.strip_prefix(root.clone()).unwrap();
                objects.push(String::from(object_path.to_str().unwrap()));
            }
        }
    }

    let ok = find_missing_elements(&mut hashes, &mut objects, RemoveSide::Both);
    dbg!(ok);
    if let Ok(entries) = utils::index::read_line(next_sync_path.clone()) {
        for entry in entries {
            // todo hash this
            staged_objects.push(String::from(entry.unwrap()));
        }
    }
    print_status(staged_objects.clone(), hashes.clone(), objects.clone());
    dbg!(hashes);
    dbg!(objects);
}

fn print_status(staged_objects: Vec<String>, hashes: HashSet<String>, objects: Vec<String>) {

    if staged_objects.len() == 0 && hashes.len() == 0 && objects.len() == 0 {
        println!("Nothing to push, working tree clean");
        return;
    }
    // staged file
    if staged_objects.len() != 0 {
        println!("Changes to be pushed:");
        println!("  (Use \"nextsync reset\" to unstage)");
        for staged in staged_objects {
            println!("      {}   {}", String::from("staged:").green(), staged.green());
        }
    }

    // not staged files
    if objects.len() != 0 || hashes.len() != 0 {
        println!("Changes not staged for push:");
        println!("  (Use\"nextsync add <file>...\" to update what will be pushed)");
    }
    for object in objects {
        println!("      {}    {}", String::from("added:").red(), object.red());
    }
    for hash in hashes {
        println!("      {}  {}", String::from("deleted:").red(), hash.red());
    }
}


fn find_missing_elements(hashes: &mut HashSet<String>, objects: &mut Vec<String>, remove_option: RemoveSide) -> Vec<String> {
    let mut hasher = Sha1::new();
    let mut to_remove: Vec<usize> = vec![];
    let mut i = 0;
    let mut duplicate = vec![];

    for object in &mut *objects {
        // hash the object
        hasher.input_str(object);
        let hash = hasher.result_str();
        hasher.reset();

        // find it on the list of hashes
        if hashes.contains(&hash) {
            duplicate.push(object.clone());
            if remove_option == RemoveSide::Left || remove_option == RemoveSide::Both {
                hashes.remove(&hash);
            }
            if remove_option == RemoveSide::Right || remove_option == RemoveSide::Both {
                to_remove.push(i);
            }
        }
        i += 1;
    }

    // remove all objects existing in the list of hashes
    i = 0;
    for index in to_remove {
        objects.remove(index-i);
        i += 1;
    }

    duplicate
}

fn is_nextsync_config(path: PathBuf) -> bool {
    path.ends_with(".nextsync") || path.ends_with(".nextsyncignore")
}

fn read_head(mut path: PathBuf) -> io::Result<io::Lines<io::BufReader<File>>> {
    path.push("HEAD");
    utils::read::read_lines(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_missing_elements() {
        let mut hasher = Sha1::new();
        hasher.input_str("file1");
        let hash1 = hasher.result_str();
        hasher.reset();
        let mut hasher = Sha1::new();
        hasher.input_str("file2");
        let hash2 = hasher.result_str();
        hasher.reset();
        let mut hasher = Sha1::new();
        hasher.input_str("file4");
        let hash4 = hasher.result_str();
        hasher.reset();

        let mut hashes = HashSet::new();
        hashes.insert(hash1.clone());
        hashes.insert(hash2.clone());
        hashes.insert(hash4.clone());

        let mut objects: Vec<String> = vec![];
        objects.push(String::from("file1"));
        objects.push(String::from("file2"));
        objects.push(String::from("file3"));
        find_missing_elements(&mut hashes, &mut objects, RemoveSide::Both);
        dbg!(hashes.clone());
        dbg!(objects.clone());
        assert_eq!(hashes.contains(&hash4), true);
        assert_eq!(hashes.len(), 1);
        assert_eq!(objects, vec!["file3"]);
    }
}
