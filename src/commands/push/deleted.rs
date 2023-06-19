use crate::services::api::ApiError;
use crate::services::req_props::ReqProps;
use crate::services::delete_path::DeletePath;
use crate::store::index;
use crate::store::object::rm_blob;
use crate::commands::status::Obj;
use crate::commands::push::push_factory::{PushState, PushChange};

pub struct Deleted {
    pub obj: Obj,
}

impl PushChange for Deleted {
    fn can_push(&self) -> PushState {
        // check if exist on server
        let res = ReqProps::new()
            .set_url(&self.obj.path.to_str().unwrap())
            .getlastmodified()
            .send_with_err();

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

        if let Ok(infos) = file_infos {
            if let Some(inf) = infos {
                // file doesn't exist on remote
                PushState::Done
            } else {
                // todo check date
                //PushState::Conflict
                PushState::Valid
            }
        } else {
            PushState::Error
        }
    }

    fn push(&self) {
        let obj = &self.obj;
        let res = DeletePath::new()
            .set_url(obj.path.to_str().unwrap())
            .send_with_err();

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
        rm_blob(&obj.path.clone());
        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }
}
