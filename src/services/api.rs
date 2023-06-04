use reqwest::Client;
use reqwest::RequestBuilder;
use reqwest::{Response, Error, IntoUrl, Method};
use std::env;
use dotenv::dotenv;

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

    pub fn set_request<U: IntoUrl>(mut self, method: Method, url: U) -> ApiBuilder {
        self.request = Some(self.client.request(method, url));
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

