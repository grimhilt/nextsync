use crate::services::api::ApiBuilder;
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
}
