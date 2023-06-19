use crate::services::api::ApiError;
use crate::services::upload_file::UploadFile;
use crate::services::delete_path::DeletePath;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::store::index;
use crate::store::object::{add_blob, rm_blob};
use crate::commands::{status, config};
use crate::commands::status::{State, Obj};
use crate::commands::push::push_factory::{PushFactory, PushState};

pub mod push_factory;
pub mod new;
pub mod deleted;

pub fn push() {
    dbg!(status::get_all_staged());

    let remote = match config::get("remote") {
        Some(r) => r,
        None => {
            eprintln!("fatal: no remote set in configuration");
            std::process::exit(1);
        }
    };
  
    let staged_objs = status::get_all_staged();
    // todo sort folder first
    for obj in staged_objs {
        if obj.otype == String::from("tree") {
           dbg!("should push folder");
        } else {
            let push_factory = PushFactory.new(obj.clone());
            match push_factory.can_push() {
                PushState::Valid => push_factory.push(),
                PushState::Done => (),
                _ => todo!(),
            }
        }
    }
    // read index
    // if dir upload dir

}
