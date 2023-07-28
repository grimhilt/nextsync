use std::path::PathBuf;
use std::fs::DirBuilder;

use crate::services::downloader::Downloader;
use crate::services::req_props::ObjProps;
use crate::store::object::blob;
use crate::store::object::tree;
use crate::utils::api::get_api_props;
use crate::utils::path;
use crate::commands::remote_diff::get_diff;


pub fn pull() {
    let relative_p = path::current()
        .unwrap()
        .strip_prefix(path::repo_root()).unwrap().to_path_buf();
    let (folders, files) = get_diff(relative_p);

    let ref_p = path::nextsync();

    for folder in folders {
        let p = ref_p.clone().join(PathBuf::from(folder.relative_s.unwrap()));
        if !p.exists() {
            // create folder
            if let Err(err) = DirBuilder::new().recursive(true).create(p.clone()) {
                eprintln!("err: cannot create directory {} ({})", p.display(), err);
            }

            // add tree
            let path_folder = p.strip_prefix(ref_p.clone()).unwrap();
            let lastmodified = folder.lastmodified.unwrap().timestamp_millis();
            if let Err(err) = tree::add(path_folder.to_path_buf(), &lastmodified.to_string(), false) {
                eprintln!("err: saving ref of {} ({})", path_folder.display(), err);
            }
        }
    }

    let downloader = Downloader::new()
        .set_api_props(get_api_props())
        .set_files(files)
        .should_log()
        .download(ref_p.clone(), Some(&update_blob));
    // todo look if need to download or update
}

fn update_blob(obj: ObjProps) {
    // todo update blob
    return;
    let relative_s = &obj.clone().relative_s.unwrap();
    let relative_p = PathBuf::from(&relative_s);
    let lastmodified = obj.clone().lastmodified.unwrap().timestamp_millis();
    if let Err(err) = blob::add(relative_p, &lastmodified.to_string(), false) {
        eprintln!("err: saving ref of {} ({})", relative_s.clone(), err);
    }
}
