use std::path::PathBuf;
use std::io;
use crate::services::api::ApiError;
use crate::services::api_call::ApiCall;
use crate::services::delete_path::DeletePath;
use crate::store::index;
use crate::store::object::blob::Blob;
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

    fn push(&self) -> io::Result<()> {
        let obj = &self.obj;
        let res = DeletePath::new()
            .set_url(obj.path.to_str().unwrap())
            .send();

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
        // todo date
        Blob::new(obj.path.clone()).rm()?;

        // remove index
        index::rm_line(obj.path.to_str().unwrap())?;

        Ok(())
    }

    fn conflict(&self) {

    }
}
