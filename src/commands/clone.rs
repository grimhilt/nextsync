use clap::Values;
use std::io::Cursor;
use xml::reader::{EventReader, XmlEvent};
use std::fs::{self, DirBuilder};
use std::path::Path;
use crate::services::list_folders::ListFolders;
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

    let mut folders = vec![url_without_domain];
    let mut url_request;
    while folders.len() > 0 {

        url_request = String::from(domain.clone());
        url_request.push_str(folders.last().unwrap().as_str());
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
        folders.pop();
        if (folders.len() == 0) {
            if DirBuilder::new().create(path.parent()).is_err() {
                // todo add second parameter to save in a folder
                eprintln!("fatal: directory already exist");
                // destination path 'path' already exists and is not an empty directory.
                std::process::exit(1);
            }

            
        }

        let objects = get_objects_xml(body);
        let mut iter = objects.iter();
        iter.next(); // jump first element which the folder fetched
        for object in iter {
            dbg!(object);
            if object.chars().last().unwrap() == '/' {
                folders.push(object.to_string());
                dbg!("folder");
            }
        }
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

