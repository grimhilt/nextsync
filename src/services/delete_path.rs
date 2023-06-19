use reqwest::{Method, Response, Error};
use crate::services::api::{ApiBuilder, ApiError};

pub struct DeletePath {
    api_builder: ApiBuilder,
}

impl DeletePath {
    pub fn new() -> Self {
        DeletePath {
            api_builder: ApiBuilder::new(),
        }
    }

    pub fn set_url(&mut self, url: &str) -> &mut DeletePath {
        self.api_builder.build_request(Method::DELETE, url);
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
