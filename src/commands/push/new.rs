use std::path::PathBuf;
use crate::services::api::ApiError;
use crate::services::req_props::{ReqProps, ObjProps};
use crate::services::upload_file::UploadFile;
use crate::store::index;
use crate::store::object::blob;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};

pub struct New {
    pub obj: LocalObj,
}

impl PushChange for New {
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState {
        match self.flow(&self.obj, whitelist.clone()) {
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

        // get lastmodified props to update it
        let props = ReqProps::new()
            .set_url(obj.path.to_str().unwrap())
            .getlastmodified()
            .send_req_single();

        let prop = match props {
            Ok(o) => o,
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: {}", err.status());
                std::process::exit(1);
            },
            Err(ApiError::EmptyError(_)) => {
                eprintln!("Failed to get body");
                std::process::exit(1);
            }
            Err(ApiError::RequestError(err)) => {
                eprintln!("fatal: {}", err);
                std::process::exit(1);
            },
            Err(ApiError::Unexpected(_)) => todo!()
        };

        let lastmodified = prop.lastmodified.unwrap().timestamp_millis();

        // update blob
        blob::add(obj.path.clone(), &lastmodified.to_string());

        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }

    // download file with .distant at the end   
    fn conflict(&self) {
        todo!()
    }
}
