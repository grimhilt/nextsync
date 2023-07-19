use crate::services::api::ApiError;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::store::object::Object;
use crate::utils::path;
use std::fs::canonicalize;
use std::path::PathBuf;

pub struct RemoteDiffArgs {
    pub path: Option<String>,
}

pub fn remote_diff(args: RemoteDiffArgs) {
    let path = {
        if let Some(path) = args.path {
            let mut cur = path::current().unwrap();
            cur.push(path);
            let canonic = canonicalize(cur).ok().unwrap();
            dbg!(&canonic);
            let ok = canonic.strip_prefix(path::repo_root());
                dbg!(&ok);
            
            PathBuf::from("/")
        } else {
            PathBuf::from("/")
        }
    };

    let mut folders: Vec<ObjProps> = vec![ObjProps {
        contentlength: None,
        href: None,
        lastmodified: None,
        relative_s: Some(path.to_str().unwrap().to_owned()),
    }];
    let mut files: Vec<ObjProps> = vec![];

    while folders.len() > 0 {
        let folder = folders.pop().unwrap();

        let res = ReqProps::new()
            .set_url(&folder.relative_s.unwrap())
            .gethref()
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



        let mut iter = objs.iter();
        // todo opti store date of root
        let root = iter.next();

        for obj in iter {
            let mut o = Object::new(&obj.clone().relative_s.unwrap());
            let exist = o.exists();

            let should_pull = {
                if exist {
                    o.read()
                        .is_older(obj.lastmodified.unwrap().timestamp())
                } else {
                    true
                }
            };
            
            if should_pull {
                println!("should pull {}", obj.clone().relative_s.unwrap());
                if obj.href.clone().unwrap().chars().last().unwrap() == '/' {
                    folders.push(obj.clone());
                } else {
                    files.push(obj.clone());
                }
            }

        }

    }
}

