use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use clap::Values;
use regex::Regex;
use crate::services::api::ApiError;
use crate::services::list_folders::ListFolders;
use crate::services::download_files::DownloadFiles;
use crate::store::object;
use crate::commands;
use crate::utils::api::{get_local_path_t, ApiProps};
use crate::global::global::{DIR_PATH, set_dir_path};

pub fn clone(remote: Values<'_>) {
    let d = DIR_PATH.lock().unwrap().clone();

    let url = remote.clone().next().unwrap();
    let (host, tmp_user, dist_path_str) = get_url_props(url);
    let username = match tmp_user {
        Some(u) => u,
        None => {
            eprintln!("No username found");
            todo!();
        }
    };
    let api_props = ApiProps {
        host: host.clone(),
        username: username.to_string(),
        root: dist_path_str.to_string(),
    };
    dbg!(dist_path_str.clone());

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

    let mut folders = vec![String::from("")];
    let mut files: Vec<String> = vec![];
    let mut first_iter = true;
    while folders.len() > 0 {
        let folder = folders.pop().unwrap();

        // request folder content
        let mut objs = vec![];
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            dbg!(folder.clone());
            let res = ListFolders::new()
                .set_request(folder.as_str(), &api_props)
                .gethref()
                .send_with_res()
                .await;
            objs = match res {
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
            }
        });

        // create folder
        if first_iter {
            if DirBuilder::new().create(ref_path.clone()).is_err() {
                eprintln!("fatal: directory already exist");
                // destination path 'path' already exists and is not an empty directory.
                //std::process::exit(1);
            } else {
                commands::init::init();
            }
        } else {
            // create folder
            let p = ref_path.clone().join(Path::new(&folder.clone()));
            if let Err(err) = DirBuilder::new().recursive(true).create(p.clone()) {
                eprintln!("error: cannot create directory {}: {}", p.display(), err);
            }

            // add tree
            let path_folder = p.strip_prefix(ref_path.clone()).unwrap();
            if object::add_tree(&path_folder).is_err() {
                eprintln!("error: cannot store object {}", path_folder.display());
            }
        }

        // find folders and files in response
        let mut iter = objs.iter();
        iter.next(); // jump first element which is the folder cloned
        for object in iter {
            dbg!(object.clone());
            if object.href.clone().unwrap().chars().last().unwrap() == '/' {
                folders.push(object.relative_s.clone().unwrap().to_string());
            } else {
                files.push(object.relative_s.clone().unwrap().to_string());
            }
        }
        first_iter = false;
    }

    download_files(ref_path.clone(), files, &api_props);
}

fn download_files(ref_p: PathBuf, files: Vec<String>, api_props: &ApiProps) {
    for relative_file in files {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = DownloadFiles::new()
                .set_url(relative_file.as_str(), api_props)
                .save(ref_p.clone()).await;

            match res {
                Ok(()) => {
                    let relative_p = Path::new(&relative_file);
                    if let Err(_) = object::add_blob(relative_p, "tmpdate") {
                        eprintln!("error saving reference of {}", relative_file.clone());
                    }
                },
                Err(ApiError::Unexpected(_)) => {
                    eprintln!("error writing {}", relative_file);
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
