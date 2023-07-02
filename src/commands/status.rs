use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Lines, BufReader};
use std::collections::{HashSet, HashMap};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use colored::Colorize;
use crate::utils::path;
use crate::utils::read::{read_folder, read_lines};
use crate::store::object::tree;
use crate::store::index;

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
// todo: not catch added empty folder
pub fn status() {
    let (mut new_objs_hashes, mut del_objs_hashes) = get_diff();
    // get copy, modified
    let mut staged_objs = get_staged(&mut new_objs_hashes, &mut del_objs_hashes);

    let mut objs: Vec<LocalObj> = del_objs_hashes.iter().map(|x| {
        x.1.clone()
    }).collect();

    for (_, elt) in new_objs_hashes {
        objs.push(elt.clone());
    }

    dbg!(objs.clone());
    dbg!(staged_objs.clone());
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
    let (mut new_objs_hashes, mut del_objs_hashes) = get_diff();
    // get copy, modified
    let mut staged_objs = get_staged(&mut new_objs_hashes, &mut del_objs_hashes);

    staged_objs.clone()
    // todo opti getting staged and then finding differences ?
}

fn get_staged(new_objs_h: &mut HashMap<String, LocalObj>, del_objs_h: &mut HashMap<String, LocalObj>) -> Vec<LocalObj> {
    let mut lines: Vec<String> = vec![];

    if let Ok(entries) = index::read_line() {
        for entry in entries {
            lines.push(entry.unwrap());
        }
    }


    let mut hasher = Sha1::new();
    let mut staged_objs: Vec<LocalObj> = vec![];

    let ref_p = path::repo_root();
    for obj in lines {
        // hash the object
        hasher.input_str(&obj);
        let hash = hasher.result_str();
        hasher.reset();

        // find it on the list of hashes
        if new_objs_h.contains_key(&hash) {
            staged_objs.push(new_objs_h.get(&hash).unwrap().clone());
            new_objs_h.remove(&hash);
        } else if del_objs_h.contains_key(&hash) {
            staged_objs.push(del_objs_h.get(&hash).unwrap().clone());
            del_objs_h.remove(&hash);
        }else {
            let mut t_path = ref_p.clone();
            let relative_p = PathBuf::from(obj.clone());
            t_path.push(relative_p.clone());
            staged_objs.push(LocalObj { 
                otype: get_otype(t_path.clone()),
                name: obj.to_string(),
                path: relative_p.clone(),
                state: {
                    if t_path.exists() {
                        State::New
                    } else {
                        State::Deleted
                    }
                },
            });
        } 
    }

    staged_objs
}

fn get_diff() -> (HashMap<String, LocalObj>, HashMap<String, LocalObj>) {
    let mut hashes = HashMap::new();
    let mut objs: Vec<String> = vec![];

    let root = path::repo_root();
      
    let nextsync_path = path::nextsync();
    let current_p = path::current().unwrap();
    let dist_path = current_p.strip_prefix(root.clone()).unwrap().to_path_buf();
    
    if let Ok(lines) = read_head(nextsync_path.clone()) {
        add_to_hashmap(lines, &mut hashes, dist_path.clone());
    }

    if let Ok(entries) = read_folder(root.clone()) {
        add_to_vec(entries, &mut objs, root.clone());
    }

    let mut obj_to_analyse = remove_duplicate(&mut hashes, &mut objs, RemoveSide::Both);

    while obj_to_analyse.len() > 0 {
        let cur_obj = obj_to_analyse.pop().unwrap();
        let cur_path = PathBuf::from(&cur_obj);

        let obj_path = root.clone().join(cur_path.clone());

        if obj_path.is_dir() {
            if let Some((_, lines)) = tree::read(cur_obj.clone()) {
                add_to_hashmap(lines, &mut hashes, cur_path.clone());
            }

            if let Ok(entries) = read_folder(obj_path.clone()) {
                add_to_vec(entries, &mut objs, root.clone());
            }

            let diff = remove_duplicate(&mut hashes, &mut objs, RemoveSide::Both);
            obj_to_analyse.append(&mut diff.clone());
        } else {
            // todo look for change
        }
            
    }

    for (_, elt) in &mut hashes {
        elt.state = State::Deleted;
    }

    let mut new_objs_hashes = HashMap::new();
    let mut hasher = Sha1::new();
    for obj in objs {
        // hash the object
        hasher.input_str(&obj);
        let hash = hasher.result_str();
        hasher.reset();
        let p = PathBuf::from(obj.to_string());
        // todo name
        new_objs_hashes.insert(String::from(hash), LocalObj {
            otype: get_otype(p.clone()),
            name: obj.to_string(),
            path: p,
            state: State::New
        });
    }

    (new_objs_hashes, hashes)
}

fn get_otype(p: PathBuf) -> String {
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
                let (ftype, hash, name) = tree::parse_line(ip);
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
    path.ends_with(".nextsync")
}

fn read_head(mut path: PathBuf) -> io::Result<io::Lines<io::BufReader<File>>> {
    path.push("HEAD");
    read_lines(path)
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

        assert_eq!(hashes.contains_key(&hash4), true);
        assert_eq!(hashes.len(), 1);
        assert_eq!(objects, vec!["file3"]);
    }
}
