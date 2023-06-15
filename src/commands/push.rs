use crate::commands::{status, config};
use crate::services::req_props::ReqProps;
use crate::services::api::ApiError;
use crate::services::upload_file::UploadFile;
use crate::commands::status::{State, Obj};

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
                _ => todo!(),
            }
        }
    }
    // read index
    // if dir upload dir

}

#[derive(Debug)]
enum PushState {
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
        let file_infos = tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = ReqProps::new()
                .set_url(&self.obj.path.to_str().unwrap())
                .getlastmodified()
                .send_with_err()
                .await;

            match res {
                Ok(data) => Ok(data),
                Err(ApiError::IncorrectRequest(err)) => {
                    if err.status() == 404 {
                        Ok(vec![])
                    } else {
                        Err(())
                    }
                },
                Err(_) => Err(()),
            }
        });

        if let Ok(infos) = file_infos {
            if infos.len() == 0 {
                // file doesn't exist on remote
                PushState::Valid
            } else {
                // check date
                PushState::Conflict
            }
        } else {
            PushState::Error
        }
    }

    fn push(&self) {
        let obj = &self.obj;
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = UploadFile::new()
                .set_url(obj.path.to_str().unwrap())
                .set_file(obj.path.clone())
                .send_with_err()
                .await;

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
            // todo manage err
            // todo remove index
        });
    }
}

struct PushFactory;

impl PushFactory {
    fn new(&self, obj: Obj) -> Box<dyn PushChange> {
        match obj.state {
            State::New => Box::new(New { obj: obj.clone() }),
            State::Renamed => todo!(),
            State::Modified => todo!(),
            State::Deleted => todo!(),
            State::Default => todo!(),
        }
    }
}

fn can_push_file(obj: Obj) -> PushState {
    dbg!(obj.clone());
    // check if exist on server
    let file_infos = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let res = ReqProps::new()
            .set_url(obj.path.to_str().unwrap())
            .getlastmodified()
            .send_with_err()
            .await;

        match res {
            Ok(data) => Ok(data),
            Err(ApiError::IncorrectRequest(err)) => {
                if err.status() == 404 {
                    Ok(vec![])
                } else {
                    Err(())
                }
            },
            Err(_) => Err(()),
        }
    });

    if let Ok(infos) = file_infos {
        if infos.len() == 0 {
            // file doesn't exist on remote
            PushState::Valid
        } else {
            // check date
           PushState::Conflict
        }
    } else {
        PushState::Error
    }
}
