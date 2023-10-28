use std::fs::File;
use std::path::PathBuf;
use std::io::{Lines, BufReader};
use std::collections::HashMap;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use colored::Colorize;
use crate::utils::path::{self, path_buf_to_string};
use crate::store::head;
use crate::store::object::blob::Blob;
use crate::utils::read::{read_folder, read_lines};
use crate::store::object::tree;
use crate::store::index;

pub struct StatusArgs {
    pub nostyle: bool,
}

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
    Moved,
    Copied,
    Modified,
    Deleted,
}

// todo: relative path, filename
// todo: not catch added empty folder
pub fn status(args: StatusArgs) {
    let mut all_hashes = get_all_objs_hashes();
    let staged_objs = get_staged(&mut all_hashes);

    let objs: Vec<LocalObj> = all_hashes.iter().map(|x| {
        x.1.clone()
    }).collect();

    if args.nostyle
    {
        print_status_nostyle(staged_objs, objs);
    }
    else
    {
        print_status(staged_objs, objs);
    }
}

pub fn get_all_objs() -> Vec<LocalObj> {
    let all_hashes = get_all_objs_hashes();
    all_hashes.iter().map(|x| {
        x.1.clone()
    }).collect()
}

fn get_all_objs_hashes() -> HashMap<String, LocalObj> {
    let (mut new_objs_hashes, mut del_objs_hashes, objs_modified) = get_diff();
    let move_copy_hashes = get_move_copy_objs(&mut new_objs_hashes, &mut del_objs_hashes);

    let mut hasher = Sha1::new();
    let mut modified_objs_hashes = HashMap::new();
    for obj in objs_modified {
        hasher.input_str(&obj);
        let hash = hasher.result_str();
        hasher.reset();

        modified_objs_hashes.insert(hash, LocalObj {
            // todo otype
            otype: get_otype(PathBuf::from(obj.clone())),
            name: obj.clone().to_string(),
            path: PathBuf::from(obj),
            path_from: None,
            state: State::Modified
        });
    }

    let mut all_hashes = HashMap::new();
    all_hashes.extend(move_copy_hashes);
    all_hashes.extend(del_objs_hashes);
    all_hashes.extend(new_objs_hashes);
    all_hashes.extend(modified_objs_hashes);

    all_hashes
}

fn should_retain(hasher: &mut Sha1, key: String, obj: LocalObj, move_copy_hashes: &mut HashMap<String, LocalObj>, del_objs_h: &mut HashMap<String, LocalObj>) -> bool {
    // todo prevent copied or moved if file empty
    // todo deal with directories
    if obj.path.is_dir()
    {
        return true;
    }
    let mut blob = Blob::new(obj.path.clone());
    let mut flag = true;
    let identical_blobs = blob.get_all_identical_blobs();

    // try to find an identical blob among the deleted files (=moved)
    for obj_s in identical_blobs.clone() {
        if !flag { break; }

        hasher.input_str(&obj_s);
        let hash = hasher.result_str();
        hasher.reset();

        if del_objs_h.contains_key(&hash) {
            let mut new_move = obj.clone();

            let deleted = del_objs_h.get(&hash).unwrap().clone();
            del_objs_h.remove(&hash);

            new_move.path_from = Some(deleted.path);
            new_move.state = State::Moved;
            move_copy_hashes.insert(key.clone(), new_move.clone());
            flag = false;
        }
    }

    // if did not find anything before try to find a file with the same content (=copy)
    if flag {
        if let Some(rel_s) = identical_blobs.first() {
            let root = path::repo_root();
            let rel_p = PathBuf::from(rel_s.clone());
            let abs_p = root.join(rel_p.clone());

            if abs_p.exists() {
                let mut new_copy = obj.clone();
                new_copy.path_from = Some(rel_p);
                new_copy.state = State::Copied;
                move_copy_hashes.insert(key, new_copy.clone());
                flag = false;
            }
        }
    }
    flag
}

fn get_move_copy_objs(new_objs_h: &mut HashMap<String, LocalObj>, del_objs_h: &mut HashMap<String, LocalObj>) -> HashMap<String, LocalObj> {
    let mut hasher = Sha1::new();
    let mut move_copy_hashes = HashMap::new();

    new_objs_h.retain(|key, obj| {
        should_retain(&mut hasher, key.to_owned(), obj.clone(), &mut move_copy_hashes, del_objs_h)
    });
    move_copy_hashes
}

#[derive(Debug, Clone)]
pub struct LocalObj {
    pub otype: String,
    pub name: String,
    pub path: PathBuf,
    pub path_from: Option<PathBuf>, // origin path when state is move or copy
    pub state: State,
}

pub fn get_all_staged() -> Vec<LocalObj> {
    let mut lines: Vec<String> = vec![];

    if let Ok(entries) = index::read_line() {
        for entry in entries {
            lines.push(entry.unwrap());
        }
    }

    let mut staged_objs = vec![];

    for line in lines {
        let obj = Blob::new(line).get_local_obj();
        if obj.state != State::Default {
            staged_objs.push(obj);
        }
    }

    staged_objs
}

fn get_staged(hashes: &mut HashMap<String, LocalObj>) -> Vec<LocalObj> {
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
        if hashes.contains_key(&hash) {
            staged_objs.push(hashes.get(&hash).unwrap().clone());
            hashes.remove(&hash);
        }else {
            let mut t_path = ref_p.clone();
            let relative_p = PathBuf::from(obj.clone());
            t_path.push(relative_p.clone());
            staged_objs.push(LocalObj { 
                otype: get_otype(t_path.clone()),
                name: obj.to_string(),
                path: relative_p.clone(),
                path_from: None,
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

fn get_diff() -> (HashMap<String, LocalObj>, HashMap<String, LocalObj>, Vec<String>) {
    let mut hashes = HashMap::new();
    let mut objs: Vec<String> = vec![];
    let mut objs_modified: Vec<String> = vec![];

    let root = path::repo_root();
      
    let current_p = path::current().unwrap();
    // todo use repo_root instead of current
    let dist_path = current_p.strip_prefix(root.clone()).unwrap().to_path_buf();
    
    if let Ok(lines) = read_lines(head::path()) {
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
            if Blob::new(cur_path).has_change() {
                objs_modified.push(cur_obj);
            }
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
        let abs_p = path::repo_root().join(p.clone());
        // todo name
        new_objs_hashes.insert(String::from(hash), LocalObj {
            otype: get_otype(abs_p),
            name: obj.to_string(),
            path: p,
            path_from: None,
            state: State::New
        });
    }

    (new_objs_hashes, hashes, objs_modified)
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
                    path_from: None,
                    state: State::Default,
                });
            }
        }
    }
}

fn add_to_vec(entries: Vec<PathBuf>, objects: &mut Vec<String>, root: PathBuf) {
    for entry in entries {
        if !path::is_nextsync_config(entry.clone()) {
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

fn print_status_nostyle(staged_objs: Vec<LocalObj>, objs: Vec<LocalObj>) {
    // todo sort
    if staged_objs.len() == 0 && objs.len() == 0 {
        return;
    }
    for obj in staged_objs {
        if obj.state == State::Deleted {
            println!("deleted: {}", obj.name);
        } else if obj.state == State::New {
            println!("new: {}", obj.name);
        } else if obj.state == State::Modified {
            println!("modified: {}", obj.name);
        } else if obj.state == State::Moved {
            println!("moved: {} => {}", path_buf_to_string(obj.path_from.unwrap()), path_buf_to_string(obj.path));
        } else if obj.state == State::Copied {
            println!("copied: {} => {}", path_buf_to_string(obj.path_from.unwrap()), path_buf_to_string(obj.path));
        }
    }
}

fn print_object(obj: LocalObj) {
    if obj.state == State::Deleted {
        println!("      {}    {}", String::from("deleted:").red(), obj.name.red());
    } else if obj.state == State::New {
        println!("      {}        {}", String::from("new:").red(), obj.name.red());
    } else if obj.state == State::Modified {
        println!("      {}   {}", String::from("modified:").red(), obj.name.red());
    } else if obj.state == State::Moved {
        println!("      {}      {} => {}", String::from("moved:").red(), path_buf_to_string(obj.path_from.unwrap()).red(), path_buf_to_string(obj.path).red());
    } else if obj.state == State::Copied {
        println!("      {}     {} => {}", String::from("copied:").red(), path_buf_to_string(obj.path_from.unwrap()), path_buf_to_string(obj.path).red());
    }
}


fn print_staged_object(obj: LocalObj) {
    if obj.state == State::Deleted {
        println!("      {}    {}", String::from("deleted:").green(), obj.name.green());
    } else if obj.state == State::New {
        println!("      {}        {}", String::from("new:").green(), obj.name.green());
    } else if obj.state == State::Modified {
        println!("      {}   {}", String::from("modified:").green(), obj.name.green());
    } else if obj.state == State::Moved {
        println!("      {}      {} => {}", String::from("moved:").green(), path_buf_to_string(obj.path_from.unwrap()).green(), path_buf_to_string(obj.path).green());
    } else if obj.state == State::Copied {
        println!("      {}     {} => {}", String::from("copied:"), path_buf_to_string(obj.path_from.unwrap()).green(), path_buf_to_string(obj.path).green());
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
            path_from: None,
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
