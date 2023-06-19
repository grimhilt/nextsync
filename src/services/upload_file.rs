use std::fs::File;
use std::io::{Read};
use std::path::PathBuf;
use reqwest::{Method, Response, Error};
use crate::services::api::{ApiBuilder, ApiError};

pub struct UploadFile {
    api_builder: ApiBuilder,
}

impl UploadFile {
    pub fn new() -> Self {
        UploadFile {
            api_builder: ApiBuilder::new(),
        }
    }

    pub fn set_url(&mut self, url: &str) -> &mut UploadFile {
        self.api_builder.build_request(Method::PUT, url);
        self
    }

    pub fn set_file(&mut self, path: PathBuf) -> &mut UploadFile {
        // todo large file
        // todo small files
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        self.api_builder.set_body(buffer);
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }

    pub fn send_with_err(&mut self) -> Result<String, ApiError> {
        let res = tokio::runtime::Runtime::new().unwrap().block_on(async {
            self.send().await 
        }).map_err(ApiError::RequestError)?;

        if res.status().is_success() {
            let body = tokio::runtime::Runtime::new().unwrap().block_on(async {
               res.text().await
            }).map_err(ApiError::EmptyError)?;
            Ok(body)
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }
}
