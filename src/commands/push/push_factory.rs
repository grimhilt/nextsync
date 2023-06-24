use std::path::Path;
use crate::commands::status::{State, LocalObj};
use crate::services::api::ApiError;
use crate::services::req_props::{ObjProps, ReqProps};
use crate::commands::push::new::New;
//use crate::commands::push::new_dir::NewDir;
//use crate::commands::push::deleted::Deleted;

#[derive(Debug)]
pub enum PushState {
    Done,
    Valid,
    Conflict,
    Error,
} 

pub enum PushFlowState {
    Whitelisted,
    NotOnRemote,
    RemoteIsNewer,
    LocalIsNewer,
    Error,
}

pub trait PushChange {
    fn can_push(&self, whitelist: Option<&Path>) -> PushState;
    fn try_push(&self, whitelist: Option<&Path>);
    fn push(&self);

    fn is_whitelisted(&self, obj: &LocalObj, path: Option<&Path>) -> bool {
        match path {
            Some(p) => obj.path.starts_with(p),
            None => false,
        }
    }

    fn flow(&self, obj: &LocalObj, whitelist: Option<&Path>) -> PushFlowState {
        if self.is_whitelisted(obj, whitelist) {
            return PushFlowState::Whitelisted;
        }

        // check if exist on server
        let res = ReqProps::new()
            .set_url(obj.path.to_str().unwrap())
            .getlastmodified()
            .send_req_single();

        let file_infos = match res {
            Ok(obj) => Ok(Some(obj)),
            Err(ApiError::IncorrectRequest(err)) => {
                if err.status() == 404 {
                    Ok(None)
                } else {
                    Err(())
                }
            },
            Err(_) => Err(()),
        };

        let infos = match file_infos {
            Ok(Some(info)) => info,
            Ok(None) => return PushFlowState::NotOnRemote,
            Err(_) => return PushFlowState::Error,
        };

        // check if remote is newest
        // set timestamp from remote stuff
        // get from file
        todo!()
    }
}

pub struct PushFactory;

impl PushFactory {
    pub fn new(&self, obj: LocalObj) -> Box<dyn PushChange> {
        match obj.state {
            State::New => Box::new(New { obj }),
            State::Renamed => todo!(),
            State::Modified => todo!(),
            State::Deleted => todo!(),
            //State::Deleted => Box::new(Deleted {}),
            State::Default => todo!(),
        }
    }

    pub fn new_dir(&self, obj: LocalObj) -> Box<dyn PushChange> {
        match obj.state {
            //State::New => Box::new(NewDir {}),
            State::New => todo!(),
            State::Renamed => todo!(),
            State::Modified => todo!(),
            State::Deleted => todo!(),
            State::Default => todo!(),
        }
    }
}


