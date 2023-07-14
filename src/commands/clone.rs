use std::io;
use std::io::prelude::*;
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use clap::Values;
use regex::Regex;
use crate::services::downloader::Downloader;
use crate::utils::api::ApiProps;
use crate::global::global::{DIR_PATH, set_dir_path};
use crate::services::api::ApiError;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::store::object::{tree, blob};
use crate::commands::config;
use crate::commands::init;

pub fn clone(remote: Values<'_>) {
    let d = DIR_PATH.lock().unwrap().clone();

    let url = remote.clone().next().unwrap();
    let (host, tmp_user, dist_path_str) = get_url_props(url);
    let username = match tmp_user {
        Some(u) => u.to_string(),
        None => {
            println!("Please enter the username of the webdav instance: ");
            let stdin = io::stdin();
            stdin.lock().lines().next().unwrap().unwrap()
        }
    };
    let api_props = ApiProps {
        host: host.clone(),
        username,
        root: dist_path_str.to_string(),
    };

    let ref_path = match d.clone() {
        Some(dir) => Path::new(&dir).to_owned(),
        None => {
            let iter = Path::new(dist_path_str).iter();
            let dest_dir = iter.last().unwrap();
            let lp = std::env::current_dir().unwrap().join(dest_dir);
            set_dir_path(lp.to_str().unwrap().to_string());
            lp
        },
    };

    let mut folders: Vec<ObjProps> = vec![ObjProps::new()];
    let mut files: Vec<ObjProps> = vec![];
    let mut first_iter = true;
    while folders.len() > 0 {
        let folder = folders.pop().unwrap();

        let relative_s = match folder.relative_s {
            Some(relative_s) => relative_s,
            None => String::from(""),
        };

        // request folder content
        let res = ReqProps::new()
            .set_request(relative_s.as_str(), &api_props)
            .gethref()
            .getcontentlength()
            .getlastmodified()
            .send_req_multiple();

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

        // create object
        if first_iter {
            // root folder, init and config
            if DirBuilder::new().recursive(true).create(ref_path.clone()).is_err() {
                eprintln!("fatal: unable to create the destination directory");
                std::process::exit(1);
            } else {
                init::init();
                let mut remote_config = api_props.username.clone();
                remote_config.push_str("@");
                remote_config.push_str(api_props.host.strip_prefix("https://").unwrap());
                remote_config.push_str(&api_props.root);
                if config::set("remote", &remote_config).is_err() {
                    eprintln!("err: not able to save remote");
                }
            }
        } else {
            // create folder
            let p = ref_path.clone().join(Path::new(&relative_s));
            if let Err(err) = DirBuilder::new().recursive(true).create(p.clone()) {
                eprintln!("err: cannot create directory {} ({})", p.display(), err);
            }

            // add tree
            let path_folder = p.strip_prefix(ref_path.clone()).unwrap();
            let lastmodified = folder.lastmodified.unwrap().timestamp_millis();
            if let Err(err) = tree::add(path_folder.to_path_buf(), &lastmodified.to_string(), false) {
                eprintln!("err: saving ref of {} ({})", path_folder.display(), err);
            }
        }

        // find folders and files in response
        let mut iter = objs.iter();
        iter.next(); // jump first element which is the folder cloned
        for object in iter {
            if object.href.clone().unwrap().chars().last().unwrap() == '/' {
                folders.push(object.clone());
            } else {
                files.push(object.clone());
            }
        }
        first_iter = false;
    }

    let downloader = Downloader::new()
        .set_api_props(api_props.clone())
        .set_files(files)
        .should_log()
        .download(ref_path.clone(), Some(&save_blob));
}

fn save_blob(obj: ObjProps) {
    let relative_s = &obj.clone().relative_s.unwrap();
    let relative_p = PathBuf::from(&relative_s);
    let lastmodified = obj.clone().lastmodified.unwrap().timestamp_millis();
    if let Err(err) = blob::add(relative_p, &lastmodified.to_string(), false) {
        eprintln!("err: saving ref of {} ({})", relative_s.clone(), err);
    }
}

pub fn get_url_props(url: &str) -> (String, Option<&str>, &str) {
    let mut username = None;
    let mut domain = "";
    let mut path = "";
    if url.find("@").is_some() {
        let re = Regex::new(r"(.*)@(.+?)(/remote\.php/dav/files/.+?)?(/.*)").unwrap();
        match re.captures_iter(url).last() {
            Some(cap) => {
                domain = cap.get(2).expect("").as_str();
                username = Some(cap.get(1).expect("").as_str());
                path = cap.get(4).expect("").as_str();
            }
            None => (),
        }
    } else if url.find("?").is_some() { // from browser url
        let re = Regex::new(r"((https?://)?.+?)/.+dir=(.+?)&").unwrap();
        match re.captures_iter(url).last() {
            Some(cap) => {
                domain = cap.get(1).expect("").as_str();
                path = cap.get(3).expect("").as_str();
            }
            None => (),
        }
    } else {
        let re = Regex::new(r"((https?://)?.+?)(/remote\.php/dav/files/(.+?))?(/.*)").unwrap();
        match re.captures_iter(url).last() {
            Some(cap) => {
                domain = cap.get(1).expect("").as_str();
                username = match cap.get(4) {
                    Some(u) => Some(u.as_str()),
                    None => None, 
                };
                path = cap.get(5).expect("").as_str();
            }
            None => (),
        }
        
    }

    let re = Regex::new(r"^http://").unwrap();
    if !re.is_match(domain) {
        let re = Regex::new(r"(^https?://)?").unwrap();
        let secure_domain = re.replace(domain, "https://").to_string();
        return (secure_domain, username, path);
    }
    (domain.to_string(), username, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_url_props() {
        let p = "/foo/bar";
        let u = Some("user");
        let d = String::from("http://nextcloud.com");
        let sd = String::from("https://nextcloud.com");
        let sld = String::from("https://nextcloud.example.com");
        let ld = String::from("http://nextcloud.example.com");
        assert_eq!(get_url_props("user@nextcloud.com/remote.php/dav/files/user/foo/bar"), (sd.clone(), u, p));
        assert_eq!(get_url_props("user@nextcloud.com/foo/bar"), (sd.clone(), u, p));
        assert_eq!(get_url_props("user@nextcloud.example.com/remote.php/dav/files/user/foo/bar"), (sld.clone(), u, p));
        assert_eq!(get_url_props("user@nextcloud.example.com/foo/bar"), (sld.clone(), u, p));
        assert_eq!(get_url_props("https://nextcloud.example.com/apps/files/?dir=/foo/bar&fileid=166666"), (sld.clone(), None, p)); 
        assert_eq!(get_url_props("https://nextcloud.com/apps/files/?dir=/foo/bar&fileid=166666"), (sd.clone(), None, p));
        assert_eq!(get_url_props("http://nextcloud.example.com/remote.php/dav/files/user/foo/bar"), (ld.clone(), u, p)); 
        assert_eq!(get_url_props("https://nextcloud.example.com/remote.php/dav/files/user/foo/bar"), (sld.clone(), u, p)); 
        assert_eq!(get_url_props("http://nextcloud.example.com/remote.php/dav/files/user/foo/bar"), (ld.clone(), u, p)); 
        assert_eq!(get_url_props("nextcloud.example.com/remote.php/dav/files/user/foo/bar"), (sld.clone(), u, p)); 
        assert_eq!(get_url_props("https://nextcloud.example.com/foo/bar"), (sld.clone(), None, p)); 
        assert_eq!(get_url_props("http://nextcloud.example.com/foo/bar"), (ld.clone(), None, p)); 
        assert_eq!(get_url_props("nextcloud.example.com/foo/bar"), (sld.clone(), None, p)); 
    }
}
