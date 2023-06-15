use xml::reader::{EventReader, XmlEvent};
use std::fs::File;
use crate::services::api::{ApiBuilder, ApiError};
use std::path::PathBuf;
use std::io::{self, Read};
use reqwest::{Method, IntoUrl, Response, Error};

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

    pub async fn send_with_err(&mut self) -> Result<String, ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.text().await.map_err(ApiError::EmptyError)?;
            Ok(body)
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }
}
