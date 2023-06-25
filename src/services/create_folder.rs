use reqwest::{Method, Response, Error};
use crate::services::api::{ApiBuilder, ApiError};

pub struct CreateFolder {
    api_builder: ApiBuilder,
}

impl CreateFolder {
    pub fn new() -> Self {
        CreateFolder {
            api_builder: ApiBuilder::new(),
        }
    }

    pub fn set_url(&mut self, url: &str) -> &mut CreateFolder {
        self.api_builder.build_request(Method::from_bytes(b"MKCOL").unwrap(), url);
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }
    
    pub fn send_with_err(&mut self) -> Result<(), ApiError> {
        let res = tokio::runtime::Runtime::new().unwrap().block_on(async {
            self.send().await
        }).map_err(ApiError::RequestError)?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }
}
