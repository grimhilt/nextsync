use std::fs::File;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::collections::HashMap;
use colored::Colorize;
use std::path::PathBuf;
use std::io::{self, Lines, BufReader};
use crate::utils::{self, object};

#[derive(PartialEq)]
enum RemoveSide {
    Left,
    Both,
    Right,
}

// todo: relative path, filename, get modified
pub fn status() {
    let (staged_objs, new_objs, del_objs) = get_diff();
    dbg!(get_diff());
    print_status(staged_objs.clone(), del_objs.iter().map(|x| x.name.to_owned()).collect(), new_objs.clone());
}

#[derive(Debug)]
pub struct Obj {
    otype: String,
    name: String,
    path: PathBuf,
}

pub fn get_diff() -> (Vec<String>, Vec<String>, Vec<Obj>) {
    let mut hashes = HashMap::new();
    let mut objs: Vec<String> = vec![];
    let mut staged_objs: Vec<String> = vec![];

    let root = match utils::path::nextsync_root() {
        Some(path) => path,
        None => {
            eprintln!("fatal: not a nextsync repository (or any of the parent directories): .nextsync");
            std::process::exit(1);
        } 
    };

    dbg!(utils::path::current());
    let nextsync_path = utils::path::nextsync().unwrap();
    let current_p = utils::path::current().unwrap();
    let dist_path = current_p.strip_prefix(root.clone()).unwrap().to_path_buf();
    
    if let Ok(lines) = read_head(nextsync_path.clone()) {
        add_to_hashmap(lines, &mut hashes, dist_path.clone());
    }

    if let Ok(entries) = utils::read::read_folder(root.clone()) {
        add_to_vec(entries, &mut objs, root.clone());
    }

    let mut obj_to_analyse = remove_duplicate(&mut hashes, &mut objs, RemoveSide::Both);
    dbg!(obj_to_analyse.clone());

    while obj_to_analyse.len() > 0 {
        let cur_obj = obj_to_analyse.pop().unwrap();
        let cur_path = PathBuf::from(&cur_obj);
        dbg!(cur_path.clone());
        let obj_path = root.clone().join(cur_path.clone());

        if obj_path.is_dir() {
            if let Some((_, lines)) = object::read_tree(cur_obj.clone()) {
                add_to_hashmap(lines, &mut hashes, cur_path.clone());
            }

            if let Ok(entries) = utils::read::read_folder(obj_path.clone()) {
                add_to_vec(entries, &mut objs, root.clone());
            }

            let diff = remove_duplicate(&mut hashes, &mut objs, RemoveSide::Both);
            obj_to_analyse.append(&mut diff.clone());
        } else {
            // todo look for change
        }
            
    }

    if let Ok(entries) = utils::index::read_line(nextsync_path.clone()) {
        for entry in entries {
            // todo hash this
            staged_objs.push(String::from(entry.unwrap()));
        }
    }

    let del_objs: Vec<Obj> = hashes.iter().map(|x| {
        Obj {otype: x.1.otype.clone(), name: x.1.name.clone(), path: x.1.path.clone()}
    }).collect();
    (staged_objs.clone(), objs.clone(), del_objs)
}

fn add_to_hashmap(lines: Lines<BufReader<File>>, hashes: &mut HashMap<String, Obj>, path: PathBuf) {
    for line in lines {
        if let Ok(ip) = line {
            if ip.clone().len() > 5 {
                let (ftype, hash, name) = object::parse_line(ip);
                let mut p = path.clone();
                p.push(name.clone());
                hashes.insert(String::from(hash), Obj{
                    otype: String::from(ftype),
                    name: String::from(name),
                    path: p,
                });
            }
        }
    }
}

fn add_to_vec(entries: Vec<PathBuf>, objects: &mut Vec<String>, root: PathBuf) {
    for entry in entries {
        if !is_nextsync_config(entry.clone()) {
            let object_path = entry.strip_prefix(root.clone()).unwrap();
            objects.push(String::from(object_path.to_str().unwrap()));
        }
    }

}

fn print_status(staged_objs: Vec<String>, del_objs: Vec<String>, new_objs: Vec<String>) {
    if staged_objs.len() == 0 && del_objs.len() == 0 && new_objs.len() == 0 {
        println!("Nothing to push, working tree clean");
        return;
    }
    // staged file
    if staged_objs.len() != 0 {
        println!("Changes to be pushed:");
        println!("  (Use \"nextsync reset\" to unstage)");
        for staged in staged_objs {
            println!("      {}   {}", String::from("staged:").green(), staged.green());
        }
    }

    // not staged files
    if new_objs.len() != 0 || del_objs.len() != 0 {
        println!("Changes not staged for push:");
        println!("  (Use\"nextsync add <file>...\" to update what will be pushed)");
    }
    for object in new_objs {
        println!("      {}    {}", String::from("added:").red(), object.red());
    }
    for object in del_objs {
        println!("      {}  {}", String::from("deleted:").red(), object.red());
    }
}


fn remove_duplicate(hashes: &mut HashMap<String, Obj>, objects: &mut Vec<String>, remove_option: RemoveSide) -> Vec<String> {
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
        if hashes.contains_key(&hash) {
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
    fn test_remove_duplicate() {
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
        remove_duplicate(&mut hashes, &mut objects, RemoveSide::Both);
        dbg!(hashes.clone());
        dbg!(objects.clone());
        assert_eq!(hashes.contains(&hash4), true);
        assert_eq!(hashes.len(), 1);
        assert_eq!(objects, vec!["file3"]);
    }
}
