use reqwest::{Method, IntoUrl, Response, Error};
use crate::services::api::{ApiBuilder, ApiError};

pub struct CreateFolder {
    api_builder: ApiBuilder,
}

impl CreateFolder {
    pub fn new<U: IntoUrl>(url: U) -> Self {
        ListFolders {
            api_builder: ApiBuilder::new()
                .set_request(Method::from_bytes(b"MKCOL").unwrap(), url),
        }
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }
    
    pub async fn send_with_err(mut self) -> Result<(), ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            Ok()
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }
}
