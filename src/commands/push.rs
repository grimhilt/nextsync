use crate::commands::{status, config};
use crate::services::req_props::{ReqProps, ObjProps};
use crate::services::api::ApiError;
use crate::services::upload_file::UploadFile;
use crate::services::delete_path::DeletePath;
use crate::commands::status::{State, Obj};
use crate::store::object::{add_blob, rm_blob};
use crate::store::index;

pub fn push() {
    dbg!(status::get_all_staged());

    let remote = match config::get("remote") {
        Some(r) => r,
        None => {
            eprintln!("fatal: no remote set in configuration");
            std::process::exit(1);
        }
    };
  
    let staged_objs = status::get_all_staged();
    // todo sort folder first
    for obj in staged_objs {
        if obj.otype == String::from("tree") {
           dbg!("should push folder");
        } else {
            let push_factory = PushFactory.new(obj.clone());
            match push_factory.can_push() {
                PushState::Valid => push_factory.push(),
                PushState::Done => (),
                _ => todo!(),
            }
        }
    }
    // read index
    // if dir upload dir

}

#[derive(Debug)]
enum PushState {
    Done,
    Valid,
    Conflict,
    Error,
} 

trait PushChange {
    fn can_push(&self) -> PushState;
    fn push(&self);
}

struct New {
    obj: Obj,
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


struct Deleted {
    obj: Obj,
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

struct PushFactory;

impl PushFactory {
    fn new(&self, obj: Obj) -> Box<dyn PushChange> {
        match obj.state {
            State::New => Box::new(New { obj: obj.clone() }),
            State::Renamed => todo!(),
            State::Modified => todo!(),
            State::Deleted => Box::new(Deleted { obj: obj.clone() }),
            State::Default => todo!(),
        }
    }
}


