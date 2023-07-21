use crate::services::api::ApiError;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::store::object::{Object, self};
use crate::utils::api::{ApiProps, get_api_props};
use crate::utils::path;
use crate::utils::remote::{enumerate_remote, EnumerateOptions};
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
            dbg!(path::repo_root());
            let ok = canonic.strip_prefix(path::repo_root());
                dbg!(&ok);
            
                // todo
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

    let depth = "2"; // todo
                     // todo origin
    let api_props = get_api_props();
    let (folders, files) = enumerate_remote(
        |a| req(&api_props, depth, a),
        &should_skip,
        EnumerateOptions {
            depth: Some(depth.to_owned()),
            relative_s: Some(path.to_str().unwrap().to_owned())
        });

    for folder in folders {
            println!("should pull {}", folder.clone().relative_s.unwrap());
    }
    for file in files {
            println!("should pull {}", file.clone().relative_s.unwrap());
    }

}

fn should_skip(obj: ObjProps) -> bool {
    let mut o = Object::new(&obj.clone().relative_s.unwrap());
    let exist = o.exists();

    // if doesn't exist locally when cannot skip it as we need to pull it
    if !exist {
        return false;
    }

    // if local directory is older there is changes on the remote we cannot
    // skip this folder
    !o.read().is_older(obj.lastmodified.unwrap().timestamp())
}

fn req(api_props: &ApiProps, depth: &str, relative_s: &str) -> Result<Vec<ObjProps>, ApiError> {
    ReqProps::new()
        .set_request(relative_s, &api_props)
        .set_depth(depth)
        .gethref()
        .getlastmodified()
        .send_req_multiple()
}

