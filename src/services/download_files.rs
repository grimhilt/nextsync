use crate::services::api::ApiBuilder;
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
}
