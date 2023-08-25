use reqwest::{Method, Response, Error, header::HeaderValue};
use crate::services::api::{ApiBuilder, ApiError};
use crate::clone::get_url_props;
use crate::commands::config;

pub struct Copy {
    api_builder: ApiBuilder,
}

impl Copy {
    pub fn new() -> Self {
        Copy {
            api_builder: ApiBuilder::new(),
        }
    }

    pub fn set_url(&mut self, url: &str, destination: &str) -> &mut Copy {
        self.api_builder.build_request(Method::from_bytes(b"COPY").unwrap(), url);
        
        let remote = match config::get("remote") {
            Some(r) => r,
            None => {
                eprintln!("fatal: unable to find a remote");
                std::process::exit(1);
            }
        };
        let (host, username, root) = get_url_props(&remote);
        let mut url = String::from(host);
        url.push_str("/remote.php/dav/files/");
        url.push_str(username.unwrap());
        url.push_str(&root);
        url.push_str("/");
        if destination !=  "/" {
            url.push_str(destination);
        } 
        self.api_builder.set_header("Destination", HeaderValue::from_str(&url).unwrap());
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }

    pub fn overwrite(&mut self, overwrite: bool) -> &mut Copy {
        self.api_builder.set_header("Overwrite", HeaderValue::from_str({
            if overwrite { "T" } else { "F" }
        }).unwrap());
        self
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
