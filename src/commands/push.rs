use crate::commands::{status, config};

pub fn push() {
    dbg!(status::get_diff());

    let remote = match config::get("remote") {
        Some(r) => r,
        None => {
            eprintln!("fatal: no remote set in configuration");
            std::process::exit(1);
        }
    };
    let (staged_obj, new_obj, del_obj) = status::get_diff();
    

}
