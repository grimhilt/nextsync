use std::path::PathBuf;
use crate::services::api::ApiError;
use crate::services::req_props::ReqProps;
use crate::services::delete_path::DeletePath;
use crate::store::index;
use crate::store::object::blob;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};

pub struct Deleted {
    pub obj: LocalObj
}

impl PushChange for Deleted {
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState {
        match self.flow(&self.obj, whitelist.clone()) {
            PushFlowState::Whitelisted => PushState::Done,
            PushFlowState::NotOnRemote => PushState::Done,
            PushFlowState::RemoteIsNewer => PushState::Conflict,
            PushFlowState::LocalIsNewer => PushState::Valid,
            PushFlowState::Error => PushState::Error,
        }
    }

    fn push(&self) {
        let obj = &self.obj;
        let res = DeletePath::new()
            .set_url(obj.path.to_str().unwrap())
            .send_with_err();

        match res {
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: error deleting file {}: {}", obj.name, err.status());
                std::process::exit(1);
            },
            Err(ApiError::RequestError(_)) => {
                eprintln!("fatal: request error deleting file {}", obj.name);
                std::process::exit(1);
            }
            _ => (),
        }

        // update tree
        blob::rm(&obj.path.clone());
        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }

    fn conflict(&self) {

    }
}
