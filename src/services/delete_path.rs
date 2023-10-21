use reqwest::Method;
use crate::services::api::{ApiBuilder, ApiError};
use crate::services::api_call::ApiCall;

pub struct DeletePath {
    api_builder: ApiBuilder,
}

impl ApiCall for DeletePath {
    fn new() -> Self {
        DeletePath {
            api_builder: ApiBuilder::new(),
        }
    }

    fn set_url(&mut self, url: &str) -> &mut DeletePath {
        self.api_builder.build_request(Method::DELETE, url);
        self
    }
    
    fn send(&mut self) -> Result<Option<String>, ApiError> {
        self.api_builder.send(true)
    }
}
