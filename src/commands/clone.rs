use std::fs::OpenOptions;
use std::fs::DirBuilder;
use std::io::prelude::*;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use clap::Values;
use regex::Regex;
use xml::reader::{EventReader, XmlEvent};
use crate::services::api::ApiError;
use crate::services::list_folders::ListFolders;
use crate::services::download_files::DownloadFiles;
use crate::store::object;
use crate::commands;
use crate::utils::api::{get_local_path, get_local_path_t};
use crate::global::global::{DIR_PATH, set_dir_path};

pub fn clone(remote: Values<'_>) {
    let d = DIR_PATH.lock().unwrap().clone();

    let url = remote.clone().next().unwrap();
    let (domain, tmp_user, dist_path_str) = get_url_props(url);
    let username = match tmp_user {
        Some(u) => u,
        None => {
            eprintln!("No username found");
            todo!();
            ""
        }
    };

    let local_path = match d.clone() {
        Some(dir) => Path::new(&dir).to_owned(),
        None => {
            let iter = Path::new(dist_path_str).iter();
            let dest_dir = iter.last().unwrap();
            let lp = std::env::current_dir().unwrap().join(dest_dir);
            set_dir_path(lp.to_str().unwrap().to_string());
            lp
        },
    };

    let mut folders = vec![String::from(dist_path_str)];
    let mut url_request;
    let mut files: Vec<String> = vec![];
    let mut first_iter = true;
    while folders.len() > 0 {
        let folder = folders.pop().unwrap();
        url_request = String::from(domain.clone());
        if first_iter {
            url_request.push_str("/remote.php/dav/files/");
            url_request.push_str(username);
        }
        url_request.push_str(folder.as_str());

        // request folder content
        let mut body = Default::default();
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            body = ListFolders::new(url_request.as_str())
                .send_with_res()
                .await;
        });

        // create folder
        if first_iter {
            if DirBuilder::new().create(local_path.clone()).is_err() {
                eprintln!("fatal: directory already exist");
                // destination path 'path' already exists and is not an empty directory.
                //std::process::exit(1);
            } else {
                commands::init::init();
            }
        } else {
            // create folder
            let local_folder = get_local_path(folder, local_path.clone(), username, dist_path_str);
            if let Err(err) = DirBuilder::new().recursive(true).create(local_folder.clone()) {
                eprintln!("error: cannot create directory {}: {}", local_folder.display(), err);
            }

            // add tree
            let path_folder = local_folder.strip_prefix(local_path.clone()).unwrap();
            if object::add_tree(&path_folder).is_err() {
                eprintln!("error: cannot store object {}", path_folder.display());
            }
        }

        // find folders and files in response
        let objects = get_objects_xml(body);
        let mut iter = objects.iter();
        iter.next(); // jump first element which is the folder cloned
        for object in iter {
            if object.chars().last().unwrap() == '/' {
                folders.push(object.to_string());
            } else {
                files.push(object.to_string());
            }
        }
        first_iter = false;
    }

    download_files(&domain, local_path.clone(), username, dist_path_str, files);
}

fn download_files(domain: &str, local_p: PathBuf, username: &str, dist_p: &str, files: Vec<String>) {
    dbg!(local_p.clone());
    for dist_file in files {
        dbg!(dist_file.clone());
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = DownloadFiles::new()
                .set_url_with_remote(dist_file.as_str())
                .save(local_p.clone()).await;

            match res {
                Ok(()) => {

                    let s = &get_local_path_t(&dist_file.clone());
                        let ss = s.strip_prefix("/").unwrap();
                    let relative_p = Path::new(ss);
                    if let Err(_) = object::add_blob(relative_p, "tmpdate") {
                        eprintln!("error saving reference of {}", dist_file.clone());
                    }
                },
                Err(ApiError::Unexpected(_)) => {
                    eprintln!("error writing {}", dist_file);
                },
                Err(ApiError::IncorrectRequest(err)) => {
                    eprintln!("fatal: {}", err.status());
                    std::process::exit(1);
                },
                Err(ApiError::EmptyError(_)) => eprintln!("Failed to get body"),
                Err(ApiError::RequestError(err)) => {
                    eprintln!("fatal: {}", err);
                    std::process::exit(1);
                }
            }
        });
    }
}

fn get_objects_xml(xml: String) -> Vec<String> {
    let cursor = Cursor::new(xml);
    let parser = EventReader::new(cursor);

    let mut should_get = false;
    let mut objects: Vec<String> = vec![];

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                should_get = name.local_name == "href";
            }
            Ok(XmlEvent::Characters(text)) => {
                if !text.trim().is_empty() && should_get {
                    objects.push(text);
                }
            }
            Ok(XmlEvent::EndElement { .. }) => {
                should_get = false;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    objects
}

fn get_url_props(url: &str) -> (String, Option<&str>, &str) {
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

