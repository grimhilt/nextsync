use crate::services::{req_props::ObjProps, api::ApiError};

pub fn enumerate_remote(req: impl Fn(&str) -> Result<Vec<ObjProps>, ApiError>, depth: Option<&str>) -> (Vec<ObjProps>, Vec<ObjProps>) {
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

        // separate folders and files in response
        
        let mut iter = objs.iter();
        // first element is not used as it is the fetched folder
        let default_depth = calc_depth(iter.next().unwrap());
        let d = depth.unwrap_or("0").parse::<u16>().unwrap();
        for object in iter {
            if object.is_dir() {
                // should get content of this folder if it is not already in this reponse
                if calc_depth(object) - default_depth == d {
                    folders.push(object.clone());
                }
                all_folders.push(object.clone());
            } else {
                files.push(object.clone());
            }
        }
    }

    (all_folders, files)
}

fn calc_depth(obj: &ObjProps) -> u16 {
    obj.relative_s.clone().unwrap_or(String::from("")).split("/").count() as u16
}

