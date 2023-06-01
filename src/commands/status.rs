use std::env;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::collections::HashSet;
use colored::Colorize;
use std::fs;

// todo: relative path, filename, get modified
pub fn status() {
    let mut hashes = HashSet::new();
    let mut objects: Vec<String> = vec![];

    let path = env::current_dir().unwrap();
    let mut next_sync_path = path.clone();
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

    if let Ok(entries) = read_folder(path.clone()) {
        for entry in entries {
            if !is_nextsync_config(entry.clone()) {
                let object_path = entry.strip_prefix(path.clone()).unwrap();
                objects.push(String::from(object_path.to_str().unwrap()));
            }
        }
    }

    find_missing_elements(&mut hashes, &mut objects);
    print_status(hashes.clone(), objects.clone());
    dbg!(hashes);
    dbg!(objects);
}

fn print_status(hashes: HashSet<String>, objects: Vec<String>) {
   if objects.len() == 0 {
       println!("No new file");
   } else {
       for object in objects {
            println!("{}    {}", String::from("Added:").green(), object.green());
       }
   }

   if hashes.len() != 0 {
       for hash in hashes {
           println!("{}  {}", String::from("Deleted:").red(), hash.red());
       }
   }
}

fn find_missing_elements(hashes: &mut HashSet<String>, objects: &mut Vec<String>) {
    let mut hasher = Sha1::new();
    let mut to_remove: Vec<usize> = vec![];
    let mut i = 0;

    for object in &mut *objects {
        // hash the object
        hasher.input_str(object);
        let hash = hasher.result_str();
        hasher.reset();

        // find it on the list of hashes
        if hashes.contains(&hash) {
            hashes.remove(&hash);
            to_remove.push(i);
        }
        i += 1;
    }

    // remove all objects existing in the list of hashes
    i = 0;
    for index in to_remove {
        objects.remove(index-i);
        i += 1;
    }
}

fn is_nextsync_config(path: PathBuf) -> bool {
    path.ends_with(".nextsync") || path.ends_with(".nextsyncignore")
}

fn read_head(mut path: PathBuf) -> io::Result<io::Lines<io::BufReader<File>>> {
    path.push("HEAD");
    read_lines(path)
}

fn read_folder(path: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
 
    entries.sort();
    Ok(entries)
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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
        find_missing_elements(&mut hashes, &mut objects);
        dbg!(hashes.clone());
        dbg!(objects.clone());
        assert_eq!(hashes.contains(&hash4), true);
        assert_eq!(hashes.len(), 1);
        assert_eq!(objects, vec!["file3"]);
    }
}
