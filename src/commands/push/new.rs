use std::path::Path;
use crate::services::api::ApiError;
use crate::services::upload_file::UploadFile;
use crate::store::index;
use crate::store::object::blob;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};

pub struct New {
    pub obj: LocalObj,
}

impl PushChange for New {
    fn can_push(&self, whitelist: Option<&Path>) -> PushState {
        match self.flow(&self.obj, whitelist) {
            PushFlowState::Whitelisted => PushState::Valid,
            PushFlowState::NotOnRemote => PushState::Valid,
            PushFlowState::RemoteIsNewer => PushState::Conflict,
            PushFlowState::LocalIsNewer => PushState::Valid,
            PushFlowState::Error => PushState::Error,
        }
    }

    fn push(&self) {
        let obj = &self.obj;
        let res = UploadFile::new()
            .set_url(obj.path.to_str().unwrap())
            .set_file(obj.path.clone())
            .send_with_err();

        match res {
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: error pushing file {}: {}", obj.name, err.status());
                std::process::exit(1);
            },
            Err(ApiError::RequestError(_)) => {
                eprintln!("fatal: request error pushing file {}", obj.name);
                std::process::exit(1);
            }
            _ => (),
        }

        // update tree
        blob::add(&obj.path.clone(), "todo_date");

        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }

    // download file with .distant at the end   
    fn conflict(&self) {
        todo!()
    }
}
