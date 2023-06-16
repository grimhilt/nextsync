use crate::services::api::{ApiBuilder, ApiError};
use std::path::PathBuf;
use reqwest::{Method, IntoUrl, Response, Error};
use crate::utils::api::get_local_path_t;
use std::fs::OpenOptions;
use std::io::{self, Write};

pub struct DownloadFiles {
    api_builder: ApiBuilder,
    path: String,
}

impl DownloadFiles {
    pub fn new() -> Self {
        DownloadFiles {
            api_builder: ApiBuilder::new(),
            path: String::from(""),
        }
    }

    pub fn set_url_with_remote(&mut self, url: &str) -> &mut DownloadFiles {
        self.path = get_local_path_t(url.clone()).strip_prefix("/").unwrap().to_string();
        self.api_builder.build_request_remote(Method::GET, url);
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }

    pub async fn send_with_err(mut self) -> Result<Vec<u8>, ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.bytes().await.map_err(ApiError::EmptyError)?;
            Ok(body.to_vec())
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }

    pub async fn save(&mut self, local_path: PathBuf) -> Result<(), ApiError> {
        let p = local_path.join(PathBuf::from(self.path.clone()));
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.bytes().await.map_err(ApiError::EmptyError)?;
            match DownloadFiles::write_file(p, &body.to_vec()) {
                Err(_) => Err(ApiError::Unexpected(String::from(""))),
                Ok(_) => Ok(()),
            }
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }

    fn write_file(path: PathBuf, content: &Vec<u8>) -> io::Result<()> {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())?;

        f.write_all(&content)?;
        Ok(())
    }
}
