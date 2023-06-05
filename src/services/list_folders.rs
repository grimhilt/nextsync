use crate::services::api::{ApiBuilder, ApiError};
use reqwest::{Method, IntoUrl, Response, Error};

pub struct ListFolders {
    api_builder: ApiBuilder,
}

impl ListFolders {
    pub fn new<U: IntoUrl>(url: U) -> Self {
        ListFolders {
            api_builder: ApiBuilder::new()
                .set_request(Method::from_bytes(b"PROPFIND").unwrap(), url),
        }
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }
    
    pub async fn send_with_err(mut self) -> Result<String, ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.text().await.map_err(ApiError::EmptyError)?;
            Ok(body)
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }

    pub async fn send_with_res(self) -> String {
        match self.send_with_err().await {
            Ok(body) => body,
            Err(ApiError::IncorrectRequest(err)) => {
                eprintln!("fatal: {}", err.status());
                std::process::exit(1);
            },
            Err(ApiError::EmptyError(_)) => {
                eprintln!("Failed to get body");
                String::from("")
            }
            Err(ApiError::RequestError(err)) => {
                eprintln!("fatal: {}", err);
                std::process::exit(1);
            }
        }
    }
}
