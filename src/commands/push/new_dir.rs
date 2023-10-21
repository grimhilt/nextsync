use std::path::PathBuf;
use std::io;
use crate::services::api::ApiError;
use crate::services::api_call::ApiCall;
use crate::services::req_props::ReqProps;
use crate::services::create_folder::CreateFolder;
use crate::store::index;
use crate::store::object::tree;
use crate::commands::status::LocalObj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFlowState};

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

    fn push(&self) -> io::Result<()> {
        let obj = &self.obj;
        let res = CreateFolder::new()
            .set_url(obj.path.to_str().unwrap())
            .send();

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

        // update tree
        tree::add(obj.path.clone(), &lastmodified.to_string(), true)?;

        // remove index
        index::rm_line(obj.path.to_str().unwrap())?;
        
        Ok(())
    }

    fn conflict(&self) {}
}
