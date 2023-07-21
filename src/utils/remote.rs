use crate::services::{req_props::ObjProps, api::ApiError};

pub fn enumerate_remote(req: impl Fn(&str) -> Result<Vec<ObjProps>, ApiError>) -> (Vec<ObjProps>, Vec<ObjProps>) {
    let mut folders: Vec<ObjProps> = vec![ObjProps::new()];
    let mut all_folders:  Vec<ObjProps> = vec![];
    let mut files: Vec<ObjProps> = vec![];

    while folders.len() > 0 {
        let folder = folders.pop().unwrap();

        let relative_s = match folder.relative_s {
            Some(relative_s) => relative_s,
            None => String::from(""),
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

        // find folders and files in response
        let mut iter = objs.iter();
        iter.next(); // jump first element which is the folder cloned
        for object in iter {
            if object.is_dir() {
                folders.push(object.clone());
                all_folders.push(object.clone());
            } else {
                files.push(object.clone());
            }
        }
    }

    (all_folders, files)
}
