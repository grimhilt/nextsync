use std::path::PathBuf;
use std::io;
use crate::services::api::ApiError;
use crate::services::r#copy::Copy;
use crate::services::api_call::ApiCall;
use crate::services::req_props::ReqProps;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};
use crate::store::object::blob::Blob;
use crate::utils::path::path_buf_to_string;

pub struct Copied {
    pub obj: LocalObj,
}

impl PushChange for Copied {
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState {
        match self.flow(&self.obj, whitelist.clone()) {
            PushFlowState::Whitelisted => PushState::Done,
            PushFlowState::NotOnRemote => PushState::Valid,
            PushFlowState::RemoteIsNewer => PushState::Conflict,
            PushFlowState::LocalIsNewer => PushState::Conflict,
            PushFlowState::Error => PushState::Error,
        }
    }

    fn push(&self) -> io::Result<()> {
        let obj = &self.obj;
        let res = Copy::new()
            .set_url_copy(
                &path_buf_to_string(obj.path_from.clone().unwrap()),
                obj.path.to_str().unwrap())
            .send();

        match res {
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: error copying file {}: {}", obj.name, err.status());
                std::process::exit(1);
            },
            Err(ApiError::RequestError(_)) => {
                eprintln!("fatal: request error copying file {}", obj.name);
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

        // create destination blob
        if let Err(err) = Blob::new(obj.path.clone()).create(&lastmodified.to_string(), false) {
            eprintln!("err: creating ref of {}: {}", obj.name.clone(), err);
        }

        Ok(())
    }

    // download file with .distant at the end   
    fn conflict(&self) {
        todo!()
    }
}
