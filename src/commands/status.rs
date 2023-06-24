use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Lines, BufReader};
use std::collections::{HashSet, HashMap};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use colored::Colorize;
use crate::utils;
use crate::store::{self, object};

#[derive(PartialEq)]
enum RemoveSide {
    Left,
    Both,
    Right,
}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    Default,
    New,
    Renamed,
    Modified,
    Deleted,
}

// todo: relative path, filename, get modified
pub fn status() {
    let (mut new_objs, mut del_objs) = get_diff();
    dbg!(get_diff());
    let mut renamed_objs = get_renamed(&mut new_objs, &mut del_objs);
    // get copy, modified
    let mut objs = new_objs;
    objs.append(&mut del_objs);
    objs.append(&mut renamed_objs);
    let staged_objs = get_staged(&mut objs);
    print_status(staged_objs, objs);
}

#[derive(Debug, Clone)]
pub struct LocalObj {
    pub otype: String,
    pub name: String,
    pub path: PathBuf,
    pub state: State,
}

pub fn get_all_staged() -> Vec<LocalObj> {
    // todo opti getting staged and then finding differences ?
    // todo opti return folder
    let (mut new_objs, mut del_objs) = get_diff();
    let mut renamed_objs = get_renamed(&mut new_objs, &mut del_objs);
    // get copy, modified
    let mut objs = new_objs;
    objs.append(&mut del_objs);
    objs.append(&mut renamed_objs);
    let staged_objs = get_staged(&mut objs);
    staged_objs
}

fn get_renamed(_new_obj: &mut Vec<LocalObj>, _del_obj: &mut Vec<LocalObj>) -> Vec<LocalObj> {
    // get hash of all new obj, compare to hash of all del
    let renamed_objs = vec![];
    renamed_objs
}

fn get_staged(objs: &mut Vec<LocalObj>) -> Vec<LocalObj> {
    let mut indexes = HashSet::new();
    let mut staged_objs: Vec<LocalObj> = vec![];

    if let Ok(entries) = store::index::read_line() {
        for entry in entries {
            indexes.insert(entry.unwrap());
        }
    }

    objs.retain(|obj| {
        if indexes.contains(obj.clone().path.to_str().unwrap()) {
            staged_objs.push(obj.clone());
            false
        } else {
            true
        }
    });

    staged_objs
}

fn get_diff() -> (Vec<LocalObj>, Vec<LocalObj>) {
    let mut hashes = HashMap::new();
    let mut objs: Vec<String> = vec![];

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

    let del_objs: Vec<LocalObj> = hashes.iter().map(|x| {
        LocalObj {
            otype: x.1.otype.clone(),
            name: x.1.name.clone(),
            path: x.1.path.clone(),
            state: State::Deleted
        }
    }).collect();

    let new_objs: Vec<LocalObj> = objs.iter().map(|x| {
        let p = PathBuf::from(x.to_string());
        // todo name
        LocalObj {
            otype: get_type(p.clone()),
            name: x.to_string(),
            path: p,
            state: State::New
        }
    }).collect();
    (new_objs, del_objs)
}

fn get_type(p: PathBuf) -> String {
    if p.is_dir() {
        String::from("tree")
    } else {
        String::from("blob")
    }
}

fn add_to_hashmap(lines: Lines<BufReader<File>>, hashes: &mut HashMap<String, LocalObj>, path: PathBuf) {
    for line in lines {
        if let Ok(ip) = line {
            if ip.clone().len() > 5 {
                let (ftype, hash, name) = object::parse_line(ip);
                let mut p = path.clone();
                p.push(name.clone());
                hashes.insert(String::from(hash), LocalObj{
                    otype: String::from(ftype),
                    name: String::from(name),
                    path: p,
                    state: State::Default,
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

fn print_status(staged_objs: Vec<LocalObj>, objs: Vec<LocalObj>) {
    dbg!(staged_objs.clone());
    dbg!(objs.clone());
    if staged_objs.len() == 0 && objs.len() == 0 {
        println!("Nothing to push, working tree clean");
        return;
    }

    // staged file
    if staged_objs.len() != 0 {
        println!("Changes to be pushed:");
        println!("  (Use \"nextsync reset\" to unstage)");
        for object in staged_objs {
            print_staged_object(object);
        }
    }

    // not staged files
    if objs.len() != 0 {
        println!("Changes not staged for push:");
        println!("  (Use\"nextsync add <file>...\" to update what will be pushed)");

        for object in objs {
            print_object(object);
        }
    }
}

fn print_object(obj: LocalObj) {
    if obj.state == State::Deleted {
        println!("      {}    {}", String::from("deleted:").red(), obj.name.red());
    } else if obj.state == State::Renamed {
        println!("      {}    {}", String::from("renamed:").red(), obj.name.red());
    } else if obj.state == State::New {
        println!("      {}        {}", String::from("new:").red(), obj.name.red());
    } else if obj.state == State::Modified {
        println!("      {}   {}", String::from("modified:").red(), obj.name.red());
    }
}

fn print_staged_object(obj: LocalObj) {
    if obj.state == State::Deleted {
        println!("      {}    {}", String::from("deleted:").green(), obj.name.green());
    } else if obj.state == State::Renamed {
        println!("      {}    {}", String::from("renamed:").green(), obj.name.green());
    } else if obj.state == State::New {
        println!("      {}        {}", String::from("new:").green(), obj.name.green());
    } else if obj.state == State::Modified {
        println!("      {}   {}", String::from("modified:").green(), obj.name.green());
    }
}

fn remove_duplicate(hashes: &mut HashMap<String, LocalObj>, objects: &mut Vec<String>, remove_option: RemoveSide) -> Vec<String> {
    let mut hasher = Sha1::new();
    let mut duplicate = vec![];

    objects.retain(|obj| {
        // hash the object
        hasher.input_str(obj);
        let hash = hasher.result_str();
        hasher.reset();

        // find it on the list of hashes
        if hashes.contains_key(&hash) {
            duplicate.push(obj.clone());

            // remove from hashes
            if remove_option == RemoveSide::Left || remove_option == RemoveSide::Both {
                hashes.remove(&hash);
            }

            // remove from objects
            remove_option != RemoveSide::Right && remove_option != RemoveSide::Both
        } else {
            true
        }
    });

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

        let mut hashes = HashMap::new();
        let default_obj = LocalObj {
            otype: String::from("tree"),
            name: String::from("test"),
            path: PathBuf::from(""),
            state: State::Default,
        };
        hashes.insert(hash1.clone(), default_obj.clone());
        hashes.insert(hash2.clone(), default_obj.clone());
        hashes.insert(hash4.clone(), default_obj.clone());

        let mut objects: Vec<String> = vec![];
        objects.push(String::from("file1"));
        objects.push(String::from("file2"));
        objects.push(String::from("file3"));
        remove_duplicate(&mut hashes, &mut objects, RemoveSide::Both);
        dbg!(hashes.clone());
        dbg!(objects.clone());
        assert_eq!(hashes.contains_key(&hash4), true);
        assert_eq!(hashes.len(), 1);
        assert_eq!(objects, vec!["file3"]);
    }
}
