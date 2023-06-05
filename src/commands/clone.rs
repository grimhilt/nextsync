use clap::Values;
use std::io::Bytes;
use std::fs::OpenOptions;
use std::env;
use std::io::prelude::*;
use std::io::Cursor;
use xml::reader::{EventReader, XmlEvent};
use std::fs::{File, DirBuilder};
use std::path::Path;
use crate::services::list_folders::ListFolders;
use crate::services::download_files::DownloadFiles;
use regex::Regex;

pub fn clone(remote: Values<'_>) {
    let url = remote.clone().next().unwrap();
    let mut path = Path::new(url);

    let domain_regex = Regex::new(r"(https?:\/\/.+?)\/").unwrap();
    let domain = match domain_regex.captures_iter(url).last() {
        Some(capture) => capture.get(1).expect("Domain not found").as_str(),
        None => {
            eprintln!("fatal: no domain found");
            std::process::exit(1);
        },
    };
    let url_without_domain = domain_regex.replace(url, "/").to_string();
    let mut it = path.iter();
    it.next();
    it.next();
    it.next();
    it.next();
    it.next();
    let username = it.next().unwrap();

    let mut folders = vec![url_without_domain];
    let mut url_request;
    let mut files: Vec<String> = vec![];
    let mut first_iter = true;
    while folders.len() > 0 {
        let folder = folders.pop().unwrap();

        url_request = String::from(domain.clone());
        url_request.push_str(folder.as_str());
        let mut body = Default::default();
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            match call(url_request.as_str()).await {
                Ok(b) => body = b.clone(),
                Err(MyError::IncorrectRequest(err)) => {
                    eprintln!("fatal: {}", err.status());
                    std::process::exit(1);
                },
                Err(MyError::EmptyError(_)) => eprintln!("Failed to get body"),
                Err(MyError::RequestError(err)) => {
                    eprintln!("fatal: {}", err);
                    std::process::exit(1);
                }
            }
        });
        if first_iter {
            first_iter = false;
            dbg!(path.file_name());
            if DirBuilder::new().create(path.file_name().unwrap()).is_err() {
                // todo add second parameter to save in a folder
                eprintln!("fatal: directory already exist");
                // destination path 'path' already exists and is not an empty directory.
                //std::process::exit(1);
            }
        } else {
            let mut path = Path::new(&folder).strip_prefix("/remote.php/dav/files/");
            path = path.unwrap().strip_prefix(username);
            DirBuilder::new().recursive(true).create(path.unwrap());
        }

        let objects = get_objects_xml(body);
        let mut iter = objects.iter();
        iter.next(); // jump first element which the folder fetched
        for object in iter {
            if object.chars().last().unwrap() == '/' {
                folders.push(object.to_string());
            } else {
                files.push(object.to_string());
            }
        }
    }

//    tokio::runtime::Runtime::new().unwrap().block_on(async {
        download_files(domain, username.to_str().unwrap(), files);
 //   });

}

fn download_files(domain: &str, username: &str, files: Vec<String>) -> std::io::Result<()> {
    dbg!("in");
    let mut body: Vec<u8> = vec![];
    for file in files {
        let mut url_request = String::from(domain.clone());
        url_request.push_str(file.as_str());
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            match callDownload(url_request.as_str()).await {
                Ok(b) => {
                    let mut path = Path::new(&file).strip_prefix("/remote.php/dav/files/");
                    path = path.unwrap().strip_prefix(username);

                    let pathCur = env::current_dir().unwrap();
                    let mut f = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(pathCur.join(path.unwrap())).unwrap();

                    f.write_all(&b);
                },
                Err(MyError::IncorrectRequest(err)) => {
                    eprintln!("fatal: {}", err.status());
                    std::process::exit(1);
                },
                Err(MyError::EmptyError(_)) => eprintln!("Failed to get body"),
                Err(MyError::RequestError(err)) => {
                    eprintln!("fatal: {}", err);
                    std::process::exit(1);
                }
            }
        });
        //f.write_all(body.clone())?;
        dbg!(file);
    }
    Ok(())
}

async fn callDownload(url: &str) -> Result<Vec<u8>, MyError> {
    let res = DownloadFiles::new(url).send().await.map_err(MyError::RequestError)?; 
    if res.status().is_success() {
        let body = res.bytes().await.map_err(MyError::EmptyError)?;
        Ok(body.to_vec())
    } else {
        Err(MyError::IncorrectRequest(res))
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

enum MyError {
    IncorrectRequest(reqwest::Response),
    EmptyError(reqwest::Error),
    RequestError(reqwest::Error),
}

async fn call(url: &str) -> Result<String, MyError> {
    let res = ListFolders::new(url).send().await.map_err(MyError::RequestError)?; 
    if res.status().is_success() {
        let body = res.text().await.map_err(MyError::EmptyError)?;
        Ok(body)
    } else {
        Err(MyError::IncorrectRequest(res))
    }
}

