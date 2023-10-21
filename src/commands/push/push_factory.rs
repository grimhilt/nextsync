use std::path::PathBuf;
use std::io;
use crate::commands::status::{State, LocalObj};
use crate::services::api::ApiError;
use crate::services::api_call::ApiCall;
use crate::services::req_props::ReqProps;
use crate::commands::push::new::New;
use crate::commands::push::new_dir::NewDir;
use crate::commands::push::rm_dir::RmDir;
use crate::commands::push::deleted::Deleted;
use crate::commands::push::modified::Modified;
use crate::commands::push::moved::Moved;
use crate::commands::push::copied::Copied;
use crate::store::object::blob::Blob;

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
    fn can_push(&self, whitelist: &mut Option<PathBuf>) -> PushState;
    fn push(&self) -> io::Result<()>;
    fn conflict(&self);

    fn is_whitelisted(&self, obj: &LocalObj, path: Option<PathBuf>) -> bool {
        match path {
            Some(p) => obj.path.starts_with(p),
            None => false,
        }
    }

    fn flow(&self, obj: &LocalObj, whitelist: Option<PathBuf>) -> PushFlowState {
        // todo moved: from same file, destination doesn't exist but parent do
        if self.is_whitelisted(obj, whitelist) {
            return PushFlowState::Whitelisted;
        }

        // check if exist on server
        let res = ReqProps::new()
            .set_url(obj.path.to_str().unwrap())
            .getlastmodified()
            .send_req_single();

        let obj_data = match res {
            Ok(obj) => Ok(Some(obj)),
            Err(ApiError::IncorrectRequest(err)) => {
                if err.status() == 404 {
                    Ok(None)
                } else {
                    eprintln!("err: when requesting properties of {} ({})", obj.name, err.status());
                    Err(())
                }
            },
            Err(_) => Err(()),
        };

        let obj_data = match obj_data {
            Ok(Some(info)) => info,
            Ok(None) => return PushFlowState::NotOnRemote,
            Err(_) => return PushFlowState::Error,
        };

        // check if remote is newest
        let last_sync_ts = Blob::new(obj.path.clone())
            .saved_remote_ts()
            .parse::<i64>().unwrap();
        let remote_ts = obj_data.lastmodified.unwrap().timestamp_millis(); 

        if last_sync_ts < remote_ts {
            PushFlowState::RemoteIsNewer
        } else {
            PushFlowState::LocalIsNewer
        }
    }
}

pub struct PushFactory;

impl PushFactory {
    pub fn new(&self, obj: LocalObj) -> Box<dyn PushChange> {
        match obj.state {
            State::New => Box::new(New { obj }),
            State::Modified => Box::new(Modified { obj }),
            State::Deleted => Box::new(Deleted { obj }),
            State::Moved => Box::new(Moved { obj }),
            State::Copied => Box::new(Copied { obj }),
            State::Default => todo!(),
        }
    }

    pub fn new_dir(&self, obj: LocalObj) -> Box<dyn PushChange> {
        match obj.state {
            State::New => Box::new(NewDir { obj }),
            State::Modified => todo!(),
            State::Deleted => Box::new(RmDir { obj }),
            State::Default => todo!(),
            _ => todo!(),
        }
    }
}


