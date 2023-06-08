use crate::commands::status;

pub fn push() {
   dbg!(status::get_diff());
   let (staged_obj, new_obj, del_obj) = status::get_diff();
}
