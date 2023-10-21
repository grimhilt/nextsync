use std::path::PathBuf;
use futures_util::StreamExt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Write};
use reqwest::{Method, Response, Error};
use crate::utils::api::ApiProps;
use crate::services::api::{ApiBuilder, ApiError};
use crate::services::api_call::ApiCall;

pub struct DownloadFiles {
    api_builder: ApiBuilder,
    relative_ps: String,
}

impl ApiCall for DownloadFiles {
    fn new() -> Self {
        DownloadFiles {
            api_builder: ApiBuilder::new(),
            relative_ps: String::new(),
        }
    }
}

impl DownloadFiles {
    // todo make it beautiful
    pub fn set_url_download(&mut self, relative_ps: &str, api_props: &ApiProps) -> &mut DownloadFiles {
        self.relative_ps = relative_ps.to_string();
        self.api_builder.set_req(Method::GET, relative_ps, api_props);
        self
    }

    pub async fn send_download(&mut self) -> Result<Response, Error> {
        self.api_builder.old_send().await
    }

    pub fn save_stream(&mut self, ref_p: PathBuf, callback: Option<impl Fn(u64)>) -> Result<(), ApiError> {
        let abs_p = ref_p.join(PathBuf::from(self.relative_ps.clone()));
        let mut file = File::create(abs_p).unwrap();

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = self.send_download().await.map_err(ApiError::RequestError)?; 
            if res.status().is_success() {
                let mut stream = res.bytes_stream();

                while let Some(chunk) = stream.next().await {
                    let unwrap_chunk = chunk.unwrap();
                    // save chunk inside file
                    if let Err(err) = file.write_all(&unwrap_chunk) {
                        return Err(ApiError::Unexpected(err.to_string()));
                    } else if let Some(fct) = &callback {
                        // call callback with size of this chunk
                        fct(unwrap_chunk.len().try_into().unwrap());
                    }
                }

                Ok(())
            } else {
                Err(ApiError::IncorrectRequest(res))
            }
        })
    }

    pub fn save(&mut self, ref_p: PathBuf) -> Result<(), ApiError> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let p = ref_p.join(PathBuf::from(self.relative_ps.clone()));
            let res = self.send_download().await.map_err(ApiError::RequestError)?; 
            if res.status().is_success() {
                let body = res.bytes().await.map_err(ApiError::EmptyError)?;
                match Self::write_file(p, &body.to_vec()) {
                    Err(_) => Err(ApiError::Unexpected(String::new())),
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
