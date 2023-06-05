use crate::services::api::{ApiBuilder, ApiError};
use reqwest::{Method, IntoUrl, Response, Error};

pub struct DownloadFiles {
    api_builder: ApiBuilder,
}

impl DownloadFiles {
    pub fn new<U: IntoUrl>(url: U) -> Self {
        DownloadFiles {
            api_builder: ApiBuilder::new()
                .set_request(Method::GET, url),
        }
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
}
