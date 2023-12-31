use std::path::PathBuf;
use std::io;
use crate::services::api::ApiError;
use crate::services::api_call::ApiCall;
use crate::services::req_props::ReqProps;
use crate::services::upload_file::UploadFile;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};
use crate::store::object::blob::Blob;

pub struct Modified {
    pub obj: LocalObj,
}

impl PushChange for Modified {
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState {
        match self.flow(&self.obj, whitelist.clone()) {
            PushFlowState::Whitelisted => PushState::Done,
            PushFlowState::NotOnRemote => PushState::Valid,
            PushFlowState::RemoteIsNewer => PushState::Conflict,
            PushFlowState::LocalIsNewer => PushState::Valid,
            PushFlowState::Error => PushState::Error,
        }
    }

    fn push(&self) -> io::Result<()> {
        let obj = &self.obj;
        let res = UploadFile::new()
            .set_url(obj.path.to_str().unwrap())
            .set_file(obj.path.clone())
            .send();

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
        Blob::new(obj.path.clone()).update(&lastmodified.to_string())?;

        Ok(())
    }

    // download file with .distant at the end   
    fn conflict(&self) {
        todo!()
    }
}
