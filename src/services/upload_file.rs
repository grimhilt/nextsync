use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use reqwest::Method;
use crate::services::api::{ApiBuilder, ApiError};
use crate::services::api_call::ApiCall;

pub struct UploadFile {
    api_builder: ApiBuilder,
}

impl ApiCall for UploadFile {
    fn new() -> Self {
        UploadFile {
            api_builder: ApiBuilder::new(),
        }
    }

    fn set_url(&mut self, url: &str) -> &mut UploadFile {
        self.api_builder.build_request(Method::PUT, url);
        self
    }

    fn send(&mut self) -> Result<Option<String>, ApiError> {
        self.api_builder.send(true)
    }
}

impl UploadFile {
    pub fn set_file(&mut self, path: PathBuf) -> &mut UploadFile {
        // todo large file
        // todo small files
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        self.api_builder.set_body(buffer);
        self
    }
}
