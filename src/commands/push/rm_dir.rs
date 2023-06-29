use std::path::PathBuf;
use crate::services::api::ApiError;
use crate::services::delete_path::DeletePath;
use crate::store::index;
use crate::store::object::tree;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};

pub struct RmDir {
    pub obj: LocalObj
}

impl PushChange for RmDir {
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState {
        match self.flow(&self.obj, whitelist.clone()) {
            PushFlowState::Whitelisted => PushState::Done,
            PushFlowState::NotOnRemote => {
                *whitelist = Some(self.obj.path.clone());
                PushState::Done
            },
            PushFlowState::RemoteIsNewer => PushState::Conflict,
            PushFlowState::LocalIsNewer => {
                *whitelist = Some(self.obj.path.clone());
                PushState::Valid
            },
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
                eprintln!("fatal: error deleting dir {}: {}", obj.name, err.status());
                std::process::exit(1);
            },
            Err(ApiError::RequestError(_)) => {
                eprintln!("fatal: request error deleting dir {}", obj.name);
                std::process::exit(1);
            }
            _ => (),
        }

        // update tree
        tree::rm(&obj.path.clone());
        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }

    fn conflict(&self) {}
}
