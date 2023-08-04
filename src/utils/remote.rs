use crate::services::{req_props::ObjProps, api::ApiError};

pub struct EnumerateOptions {
    pub depth: Option<String>,
    pub relative_s: Option<String>,
}

pub fn enumerate_remote(
    req: impl Fn(&str) -> Result<Vec<ObjProps>, ApiError>,
    should_skip: &dyn Fn(ObjProps) -> bool,
    options: EnumerateOptions
    ) -> (Vec<ObjProps>, Vec<ObjProps>) {

    let mut folders: Vec<ObjProps> = vec![ObjProps::new()];
    let mut all_folders:  Vec<ObjProps> = vec![];
    let mut files: Vec<ObjProps> = vec![];

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
        let mut iter = objs.iter();
        // first element is not used as it is the fetched folder
        let default_depth = calc_depth(iter.next().unwrap());
        let d = options.depth.clone().unwrap_or("0".to_owned()).parse::<u16>().unwrap();
        let mut skip_depth = 0;
        for object in iter {
            if object.is_dir() {
                let current_depth = calc_depth(object);
                // skip children of skiped folder
                if skip_depth != 0 && skip_depth < current_depth {
                   continue; 
                }

                let should_skip = should_skip(object.clone());
                if should_skip {
                    skip_depth = current_depth;
                } else {
                    skip_depth = 0;
                    all_folders.push(object.clone());
                }

                // should get content of this folder if it is not already in this reponse
                if current_depth - default_depth == d && !should_skip {
                    folders.push(object.clone());
                }
            } else {
                let current_depth = calc_depth(object);
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

    (all_folders, files)
}

fn calc_depth(obj: &ObjProps) -> u16 {
    obj.relative_s.clone().unwrap_or(String::new()).split("/").count() as u16
}

