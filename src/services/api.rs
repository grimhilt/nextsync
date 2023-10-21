use std::env;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::path::PathBuf;
use dotenv::dotenv;
use reqwest::Client;
use reqwest::RequestBuilder;
use reqwest::multipart::Form;
use reqwest::{Response, Error, Method};
use reqwest::header::{HeaderValue, CONTENT_TYPE, HeaderMap, IntoHeaderName};
use crate::utils::api::ApiProps;
use crate::commands::config;
use crate::commands::clone::get_url_props;
use crate::services::request_manager::get_request_manager;
use crate::services::api_call::ApiCall;

use super::login::Login;

lazy_static! {
    static ref HTTP_TOKEN: Mutex<String> = Mutex::new(String::new());
}

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
    headers: Option<HeaderMap>,
    auth_set: bool,
    host: Option<String>,
}

impl ApiBuilder {
    pub fn new() -> Self {
        ApiBuilder {
            client: Client::new(),
            request: None,
            headers: None,
            auth_set: false,
            host: None,
        }
    }

    pub fn set_url(&mut self, method: Method, url: &str) -> &mut ApiBuilder {
        self.request = Some(self.client.request(method, url));
        self
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
        self.host = Some(host.clone());
        let mut url = String::from(host);
        url.push_str("/remote.php/dav/files/");
        url.push_str(username.unwrap());
        url.push_str(&root);
        url.push_str("/");
        if path !=  "/" {
            url.push_str(path);
        } 
        self.request = Some(self.client.request(method, url));
        self
    }

    pub fn set_req(&mut self, meth: Method, p: &str, api_props: &ApiProps) -> &mut ApiBuilder {
        self.host = Some(api_props.clone().host.clone());
        let mut url = String::from(&api_props.host);
        url.push_str("/remote.php/dav/files/");
        url.push_str("/");
        url.push_str(&api_props.username);
        url.push_str(&api_props.root);
        url.push_str("/");
        if p !=  "/" {
            url.push_str(p);
        } 
        self.request = Some(self.client.request(meth, url));
        self
    }

    pub fn set_basic_auth(&mut self, login: String, pwd: String) -> &mut ApiBuilder {
        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => {
                self.request = Some(req.basic_auth(login, Some(pwd)));
            }
        }
        self.auth_set = true;
        self
    }

    fn set_auth(&mut self) -> &mut ApiBuilder {
        // check .config
        //let config_file = PathBuf::from("~/.nextsync/config");
        //if config_file.exists() {
        //    
        //} else {
        //    let res = Login::new()
        //        .set_host(self.host.clone())
        //        .ask_auth()
        //        .send_with_err();

        //    if let Err(err) = res {
        //        eprintln!("fatal: authentification failed");
        //        std::process::exit(1);
        //    }

        //}
        //// todo if not exist
        //dotenv().ok();
        //let password = env::var("PASSWORD").unwrap();
        //let username = env::var("USERNAME").unwrap();
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
                self.set_header(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
            }
        }
        self
    }

    pub fn set_multipart(&mut self, form: Form) -> &mut ApiBuilder {
        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => {
                self.request = Some(req.multipart(form));
                self.set_header(CONTENT_TYPE, HeaderValue::from_static("multipart/related"));
            }
        }
        self
    }

    pub fn set_header<K: IntoHeaderName>(&mut self, key: K, val: HeaderValue) -> &mut ApiBuilder {
        let map = self.headers.get_or_insert(HeaderMap::new());
        map.insert(key, val);
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

    pub fn send(&mut self, need_text: bool) -> Result<Option<String>, ApiError> {
        let mut request_manager = get_request_manager().lock().unwrap();
        let request_manager = request_manager.as_mut().unwrap();
        if !self.host.is_none()
        {
            request_manager.set_host(self.host.clone().unwrap());
        }

        if !self.auth_set {
            //self.set_auth();
            self.set_header("TOKEN", HeaderValue::from_str(&request_manager.get_token()).unwrap());
        }


        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = match self.request.take() {
                None => {
                    eprintln!("fatal: incorrect request");
                    std::process::exit(1);
                },
                Some(req) => {
                    if let Some(headers) = &self.headers {
                        req.headers(headers.clone())
                            .send().await.map_err(ApiError::RequestError)?
                    } else {
                        req.send().await.map_err(ApiError::RequestError)?
                    }
                },
            };
            
            if res.status().is_success() {
                if need_text {
                    let body = res.text().await.map_err(|err| ApiError::EmptyError(err))?;
                    Ok(Some(body))
                } else {
                    Ok(None)
                }
            } else {
                Err(ApiError::IncorrectRequest(res))
            }
        })
    }

    pub async fn old_send(&mut self) -> Result<Response, Error> {
        let mut request_manager = get_request_manager().lock().unwrap();
        let request_manager = request_manager.as_mut().unwrap();
        if !self.host.is_none()
        {
            request_manager.set_host(self.host.clone().unwrap());
        }

        if !self.auth_set {
            //self.set_auth();
            self.set_header("TOKEN", HeaderValue::from_str(&request_manager.get_token()).unwrap());
        }


        match self.request.take() {
            None => {
                eprintln!("fatal: incorrect request");
                std::process::exit(1);
            },
            Some(req) => {
                if let Some(headers) = &self.headers {
                    req.headers(headers.clone())
                        .send().await.map_err(Error::from)
                } else {
                    req.send().await.map_err(Error::from)
                }
            },
        }
    }
}

