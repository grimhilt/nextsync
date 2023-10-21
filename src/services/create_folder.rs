use reqwest::Method;
use crate::services::api::{ApiBuilder, ApiError};
use crate::services::api_call::ApiCall;

pub struct CreateFolder {
    api_builder: ApiBuilder,
}

impl ApiCall for CreateFolder {
    fn new() -> Self {
        CreateFolder {
            api_builder: ApiBuilder::new(),
        }
    }

    fn set_url(&mut self, url: &str) -> &mut CreateFolder {
        self.api_builder.build_request(Method::from_bytes(b"MKCOL").unwrap(), url);
        self
    }

    fn send(&mut self) -> Result<Option<String>, ApiError> {
        self.api_builder.send(false)
    }
}
