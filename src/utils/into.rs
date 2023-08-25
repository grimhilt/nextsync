use std::path::PathBuf;

pub trait IntoPathBuf {
    fn into(self) -> PathBuf;
}

impl IntoPathBuf for PathBuf {
    fn into(self) -> PathBuf {
        self
    }
}

impl IntoPathBuf for String {
    fn into(self) -> PathBuf {
        PathBuf::from(self)
    }
}

