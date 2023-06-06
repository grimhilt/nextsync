use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DIR_PATH: Mutex<Option<String>> = Mutex::new(None);
}

pub fn set_dir_path(path: String) {
    let mut directory_path = DIR_PATH.lock().unwrap();
    *directory_path = Some(path);
}
