use reqwest::{Method, header::HeaderValue};
use crate::services::api::{ApiBuilder, ApiError};
use crate::commands::clone::get_url_props;
use crate::commands::config;
use crate::services::api_call::ApiCall;

pub struct Copy {
    api_builder: ApiBuilder,
}

impl ApiCall for Copy {
    fn new() -> Self {
        Copy {
            api_builder: ApiBuilder::new(),
        }
    }

    fn send(&mut self) -> Result<Option<String>, ApiError> {
        self.api_builder.send(true)
    }
}

impl Copy {
    pub fn set_url_copy(&mut self, url: &str, destination: &str) -> &mut Copy {
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

    pub fn _overwrite(&mut self, overwrite: bool) -> &mut Copy {
        self.api_builder.set_header("Overwrite", HeaderValue::from_str({
            if overwrite { "T" } else { "F" }
        }).unwrap());
        self
    }
}
