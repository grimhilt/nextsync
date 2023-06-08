use crate::commands::{status, config};

pub fn push() {
   dbg!(status::get_diff());
   let (staged_obj, new_obj, del_obj) = status::get_diff();
   dbg!(config::get("remote"));
}
