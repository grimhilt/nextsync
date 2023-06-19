use crate::services::api::ApiError;
use crate::services::req_props::ReqProps;
use crate::services::upload_file::UploadFile;
use crate::store::index;
use crate::store::object::add_blob;
use crate::commands::status::Obj;
use crate::commands::push::push_factory::{PushState, PushChange, PushFactory};

pub struct New {
    pub obj: Obj,
}

impl PushChange for New {
    fn can_push(&self) -> PushState {
        // check if exist on server
        let res = ReqProps::new()
            .set_url(&self.obj.path.to_str().unwrap())
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

        if let Ok(infos) = file_infos {
            if let Some(info) = infos {
                // file doesn't exist on remote
                PushState::Valid
            } else {
                // todo check date
                PushState::Conflict
            }
        } else {
            PushState::Error
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
        add_blob(&obj.path.clone(), "todo_date");

        // remove index
        index::rm_line(obj.path.to_str().unwrap());
    }
}
