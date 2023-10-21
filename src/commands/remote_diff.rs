use crate::services::api::ApiError;
use crate::services::api_call::ApiCall;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::store::object::Object;
use crate::utils::api::{ApiProps, get_api_props};
use crate::utils::path;
use crate::utils::remote::{enumerate_remote, EnumerateOptions};
use std::path::PathBuf;

// todo deletion
pub fn remote_diff() {
    let relative_p = path::current()
        .unwrap()
        .strip_prefix(path::repo_root()).unwrap().to_path_buf();
    let (folders, files) = get_diff(relative_p);

    for folder in folders {
        println!("should pull {}", folder.clone().relative_s.unwrap());
    }
    for file in files {
        println!("should pull {}", file.clone().relative_s.unwrap());
    }
}

pub fn get_diff(path: PathBuf) -> (Vec<ObjProps>, Vec<ObjProps>) {

    let depth = "2"; // todo opti
    let api_props = get_api_props();

    enumerate_remote(
        |a| req(&api_props, depth, a),
        Some(&should_skip),
        EnumerateOptions {
            depth: Some(depth.to_owned()),
            relative_s: Some(path.to_str().unwrap().to_owned())
        })
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

