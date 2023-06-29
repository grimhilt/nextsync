use std::env;
use dotenv::dotenv;
use reqwest::Client;
use reqwest::RequestBuilder;
use reqwest::{Response, Error, Method};
use crate::utils::api::ApiProps;
use crate::commands::config;
use crate::commands::clone::get_url_props;

#[derive(Debug)]
pub enum ApiError {
    IncorrectRequest(reqwest::Response),
    EmptyError(reqwest::Error),
    RequestError(reqwest::Error),
    Unexpected(String),
}

pub struct ApiBuilder {
    client: Client,
    request: Option<RequestBuilder>,
}

impl ApiBuilder {
    pub fn new() -> Self {
        ApiBuilder {
            client: Client::new(),
            request: None,
        }
    }

    pub fn build_request(&mut self, method: Method, path: &str) -> &mut ApiBuilder {
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
        url.push_str(path);
        self.request = Some(self.client.request(method, url));
        self
    }

    pub fn set_req(&mut self, meth: Method, p: &str, api_props: &ApiProps) -> &mut ApiBuilder {
        let mut url = String::from(&api_props.host);
        url.push_str("/remote.php/dav/files/");
        url.push_str("/");
        url.push_str(&api_props.username);
        url.push_str(&api_props.root);
        url.push_str("/");
        url.push_str(p);
        self.request = Some(self.client.request(meth, url));
        self
    }

    fn set_auth(&mut self) -> &mut ApiBuilder {
        // todo if not exist
        dotenv().ok();
        let password = env::var("PASSWORD").unwrap();
        let username = env::var("USERNAME").unwrap();
        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => {
                self.request = Some(req.basic_auth(username, Some(password)));
            }
        }
        self
    }

    pub fn set_xml(&mut self, xml_payload: String) -> &mut ApiBuilder {
        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => {
                self.request = Some(req.body(xml_payload));
            }
        }
        self
    }

    pub fn set_body(&mut self, body: Vec<u8>) -> &mut ApiBuilder {
        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => {
                self.request = Some(req.body(body));
            }
        }
        self

    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.set_auth();
        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => req.send().await.map_err(Error::from),
        }
    }
}

