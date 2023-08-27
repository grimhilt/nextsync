use std::path::PathBuf;
use crate::{services::{req_props::{ObjProps, ReqProps}, api::ApiError}, store::object::{blob::Blob, Object}, commands::status::State};
use std::collections::HashMap;

use super::{path::{path_buf_to_string, self}, read};

pub struct EnumerateOptions {
    pub depth: Option<String>,
    pub relative_s: Option<String>,
}

pub fn enumerate_remote(
    req: impl Fn(&str) -> Result<Vec<ObjProps>, ApiError>,
    should_skip: Option<&dyn Fn(ObjProps) -> bool>,
    options: EnumerateOptions
    ) -> (Vec<ObjProps>, Vec<ObjProps>) {

    let mut folders: Vec<ObjProps> = vec![ObjProps::new()];
    let mut all_folders:  Vec<ObjProps> = vec![];
    let mut deleted:  Vec<PathBuf> = vec![];
    let mut files: Vec<ObjProps> = vec![];
    let mut objs_hashmap: HashMap<String, Vec<String>> = HashMap::new();
    objs_hashmap.insert(
        options.relative_s.clone().unwrap_or(String::new()),
        Vec::new());

    while folders.len() > 0 {
        let folder = folders.pop().unwrap();

        let relative_s = match folder.relative_s {
            Some(relative_s) => relative_s,
            None => options.relative_s.clone().unwrap_or(String::new())
        };

        // request folder content
        let res = req(relative_s.as_str());

        let objs = match res {
            Ok(o) => o,
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: {}", err.status());
                std::process::exit(1);
            },
            Err(ApiError::EmptyError(_)) => {
                eprintln!("Failed to get body");
                vec![]
            }
            Err(ApiError::RequestError(err)) => {
                eprintln!("fatal: {}", err);
                std::process::exit(1);
            },
            Err(ApiError::Unexpected(_)) => todo!()
        };

        // separate folders and files in response
        let d = options.depth.clone().unwrap_or("0".to_owned()).parse::<u16>().unwrap();
        // first element is not used as it is the fetched folder
        if let Some(should_skip_fct) = should_skip.clone() {
            iter_with_skip_fct(
                objs,
                d,
                &mut files,
                &mut folders,
                should_skip_fct,
                &mut objs_hashmap,
                &mut all_folders);

            // check for deletion only when folder are not empty
            // as the folder's content may not have been fetched yet
            for (key, children) in objs_hashmap.clone() {
                if children.len() != 0 {
                    get_deleted(key.clone(), children, &mut deleted);
                    objs_hashmap.remove(&key);
                }
            }
        } else {
            iter_without_skip_fct(
                objs,
                d,
                &mut files,
                &mut folders,
                &mut all_folders);
        }
    }
    // go through all folders not checked for deletion before
    // as they were empty
    if let Some(_) = should_skip.clone() {
        for (key, children) in objs_hashmap.clone() {
            get_deleted(key.clone(), children, &mut deleted);
            objs_hashmap.remove(&key);
        }
    }
    dbg!(deleted);
    dbg!(objs_hashmap);

    (all_folders, files)
}

fn calc_depth(obj: &ObjProps) -> u16 {
    calc_depth_string(obj.relative_s.clone().unwrap_or(String::new()))
}

fn calc_depth_string(s: String) -> u16 {
    s.split("/").count() as u16
}

fn iter_with_skip_fct(
    objs: Vec<ObjProps>,
    d: u16,
    files: &mut Vec<ObjProps>,
    folders: &mut Vec<ObjProps>,
    should_skip: &dyn Fn(ObjProps) -> bool,
    objs_hashmap: &mut HashMap<String, Vec<String>>,
    all_folders: &mut Vec<ObjProps>) {

    let mut iter = objs.iter();
    let default_depth = calc_depth(iter.next().unwrap());
    let mut skip_depth = 0;

    for object in iter {
        let current_depth = calc_depth(object);

        if object.is_dir() {

            // add folder to parent folder only if exists
            let mut r_path = PathBuf::from(object.relative_s.clone().unwrap());
            r_path.pop();
            let r_ps = path_buf_to_string(r_path);
            if let Some(values) = objs_hashmap.get_mut(&r_ps.clone()) {
                values.push(object.relative_s.clone().unwrap());
            }
            
            // skip children of skiped folder
            if skip_depth != 0 && skip_depth < current_depth {
                continue; 
            }

            let should_skip = should_skip(object.clone());
            if should_skip {
                skip_depth = current_depth;
            } else {
                // if this folder is not skipped then we initialised its vector
                let r_ps_dir = object.relative_s.clone().unwrap();
                let mut r_ps_key = r_ps_dir.chars();
                r_ps_key.next_back();
                objs_hashmap.insert(r_ps_key.as_str().to_owned(), Vec::new());

                skip_depth = 0;
                all_folders.push(object.clone());
            }

            // should get content of this folder if it is not already in this reponse
            if current_depth - default_depth == d && !should_skip {
                folders.push(object.clone());
            }
        } else {
            // add file to parent folder only if exists
            let mut r_path = PathBuf::from(object.relative_s.clone().unwrap());
            r_path.pop();
            let r_ps = path_buf_to_string(r_path);
            if let Some(values) = objs_hashmap.get_mut(&r_ps.clone()) {
                values.push(object.relative_s.clone().unwrap());
            }
            
            // skip children of skiped folder
            if skip_depth != 0 && skip_depth < current_depth {
                continue; 
            }

            if !should_skip(object.clone()) {
                skip_depth = 0;
                files.push(object.clone());
            }
        }
    }
}

fn iter_without_skip_fct(
    objs: Vec<ObjProps>,
    d: u16,
    files: &mut Vec<ObjProps>,
    folders: &mut Vec<ObjProps>,
    all_folders: &mut Vec<ObjProps>) {

    let mut iter = objs.iter();
    let default_depth = calc_depth(iter.next().unwrap());

    for object in iter {
        if object.is_dir() {
            // should get content of this folder if it is not already in this reponse
            let current_depth = calc_depth(object);
            if current_depth - default_depth == d {
                folders.push(object.clone());
            }
            all_folders.push(object.clone());
        } else {
            files.push(object.clone());
        }
    }

}

fn get_non_new_local_element(iter: &mut dyn Iterator<Item = &PathBuf>) -> Option<PathBuf> {
    let mut el = iter.next();
    while !el.is_none() && {
        if el.unwrap().is_dir() {
            // ignore newly created directory (not sync)
            !Object::new(el.unwrap().clone().to_str().unwrap()).exists()
        } else {
            // ignore newly created file (not sync)
            Blob::new(el.unwrap().clone()).status(&mut None) == State::New 
        } 
    } {
        el = iter.next();
    }
    match el {
        Some(e) => Some(e.to_owned()),
        None => None
    }
}

fn get_deleted(source: String, children: Vec<String>, deleted: &mut Vec<PathBuf>) {
    let root = path::repo_root();
    let abs_p = root.join(PathBuf::from(source.clone()));

    let folder_read = read::read_folder(abs_p.clone());
    if let Ok(mut local_objs) = folder_read {
        // set path to be ref one not abs
        local_objs.iter_mut().for_each(|e| {
            *e = e.strip_prefix(path_buf_to_string(root.clone())).unwrap().to_path_buf();
        });

        let mut iter = local_objs.iter();
        let mut local_element = get_non_new_local_element(&mut iter);

        while let Some(local) = local_element {
            if let None = children.iter().position(|child| {
                let child_compared = {
                    // remove traling / of directory
                    if child.ends_with("/") {
                        let t = child.clone();
                        let mut ts = t.chars();
                        ts.next_back();
                        ts.as_str().to_owned()
                    } else {
                        child.clone()
                    }
                };

                child_compared == path_buf_to_string(local.clone())
            }) {
                deleted.push(local.clone());
            }
            local_element = get_non_new_local_element(&mut iter);
        }
    }
}

