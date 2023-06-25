use std::path::PathBuf;
use crate::services::api::ApiError;
use crate::services::req_props::ReqProps;
use crate::services::create_folder::CreateFolder;
use crate::store::index;
use crate::store::object::tree;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFactory, PushFlowState};

pub struct NewDir {
    pub obj: LocalObj
}

impl PushChange for NewDir {
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState {
        match self.flow(&self.obj, whitelist.clone()) {
            PushFlowState::Whitelisted => PushState::Valid,
            PushFlowState::NotOnRemote => {
                *whitelist = Some(self.obj.path.clone());
                PushState::Valid
            },
            PushFlowState::RemoteIsNewer => PushState::Conflict,
            PushFlowState::LocalIsNewer => {
                *whitelist = Some(self.obj.path.clone());
                PushState::Done
            },
            PushFlowState::Error => PushState::Error,
        }
    }

    fn push(&self) {
        let obj = &self.obj;
        let res = CreateFolder::new()
            .set_url(obj.path.to_str().unwrap())
            .send_with_err();

        match res {
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: error creating folder {}: {}", obj.name, err.status());
                std::process::exit(1);
            },
            Err(ApiError::RequestError(_)) => {
                eprintln!("fatal: request error creating folder {}", obj.name);
                std::process::exit(1);
            }
            _ => (),
        }

        // update tree
        tree::add(&obj.path.clone(), "todo_date");

        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }

    fn conflict(&self) {}
}
