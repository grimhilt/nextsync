use std::path::PathBuf;
use crate::services::api::ApiError;
use crate::services::upload_file::UploadFile;
use crate::services::delete_path::DeletePath;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::store::index;
use crate::store::object::blob;
use crate::commands::{status, config};
use crate::commands::status::{State, LocalObj};
use crate::commands::push::push_factory::{PushFactory, PushState};

pub mod push_factory;
pub mod new;
pub mod new_dir;
pub mod deleted;

pub fn push() {
    dbg!(status::get_all_staged());

    let remote = match config::get("remote") {
        Some(r) => r,
        None => {
            eprintln!("fatal: no remote set in configuration");
            //std::process::exit(1);
            String::from("")
        }
    };
  
    let staged_objs = status::get_all_staged();
    // todo sort folder first

    // path that certify that all its children can be push whithout hesistation
    // (e.g if remote dir has no changes since last sync all children
    // can be pushed without verification)
    let mut whitelist: Option<PathBuf> = None;

    for obj in staged_objs {
        if obj.otype == String::from("tree") {
            let push_factory = PushFactory.new_dir(obj.clone());
            let res = match push_factory.can_push(&mut whitelist) {
                PushState::Valid => push_factory.push(),
                PushState::Done => (),
                PushState::Conflict => (),
                _ => todo!(),
            };

            dbg!("should push folder");
        } else {
            let push_factory = PushFactory.new(obj.clone());
            match push_factory.can_push(&mut whitelist) {
                PushState::Valid => push_factory.push(),
                PushState::Done => (),
                PushState::Conflict => {
                    // download file
                }
                _ => todo!(),
            }
        }
    }
    // read index
    // if dir upload dir
}
