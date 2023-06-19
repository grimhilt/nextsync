use crate::services::api::{ApiBuilder, ApiError};
use std::path::PathBuf;
use reqwest::{Method, Response, Error};
use crate::utils::api::ApiProps;
use std::fs::OpenOptions;
use std::io::{self, Write};

pub struct DownloadFiles {
    api_builder: ApiBuilder,
    relative_ps: String,
}

impl DownloadFiles {
    pub fn new() -> Self {
        DownloadFiles {
            api_builder: ApiBuilder::new(),
            relative_ps: String::from(""),
        }
    }

    pub fn set_url(&mut self, relative_ps: &str, api_props: &ApiProps) -> &mut DownloadFiles {
        self.relative_ps = relative_ps.to_string();
        self.api_builder.set_req(Method::from_bytes(b"PROPFIND").unwrap(), relative_ps, api_props);
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }

    pub async fn _send_with_err(mut self) -> Result<Vec<u8>, ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.bytes().await.map_err(ApiError::EmptyError)?;
            Ok(body.to_vec())
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }

    pub fn save(&mut self, ref_p: PathBuf) -> Result<(), ApiError> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let p = ref_p.join(PathBuf::from(self.relative_ps.clone()));
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
        })
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
