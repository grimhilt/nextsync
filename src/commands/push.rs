use std::path::PathBuf;
use crate::commands::{status, config};
use crate::commands::push::push_factory::{PushFactory, PushState};

pub mod push_factory;
pub mod new;
pub mod new_dir;
pub mod rm_dir;
pub mod deleted;

pub fn push() {
    // todo err when pushing new folder
    // todo
    let _remote = match config::get("remote") {
        Some(r) => r,
        None => {
            eprintln!("fatal: no remote set in configuration");
            //std::process::exit(1);
            String::new()
        }
    };
  
    let staged_objs = status::get_all_staged();

    // path that certify that all its children can be push whithout hesistation
    // (e.g. if remote dir has no changes since last sync all children
    // can be pushed without verification)
    let mut whitelist: Option<PathBuf> = None;

    for obj in staged_objs {
        if obj.otype == String::from("tree") {
            let push_factory = PushFactory.new_dir(obj.clone());
            let res = push_factory.can_push(&mut whitelist);
            match res {
                PushState::Valid => {
                    match push_factory.push() {
                        Ok(()) => (),
                        Err(err) => {
                            eprintln!("err: pushing {}: {}", obj.name, err);
                        }
                    }
                },
                PushState::Done => (),
                PushState::Conflict => {
                    println!("CONFLICT: {}", obj.clone().name);
                },
                _ => todo!(),
            };

        } else {
            let push_factory = PushFactory.new(obj.clone());
            match push_factory.can_push(&mut whitelist) {
                PushState::Valid => {
                    match push_factory.push() {
                        Ok(()) => (),
                        Err(err) => {
                            eprintln!("err: pushing {}: {}", obj.name, err);
                        }
                    }
                },
                PushState::Done => (),
                PushState::Conflict => {
                    // download file
                }
                _ => todo!(),
            }
        }
    }
    // read index
    // if dir upload dir
}
