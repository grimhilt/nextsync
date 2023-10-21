use std::io;
use std::io::Cursor;
use std::io::prelude::*;
use xml::reader::{EventReader, XmlEvent};
use reqwest::{Response, Error, header::HeaderValue, Method};
use rpassword;
use crate::services::api_call::ApiCall;

use crate::services::api::{ApiBuilder, ApiError};

pub struct Login {
    api_builder: ApiBuilder,
    login: String,
    password: String,
    host: Option<String>,
}

impl ApiCall for Login {
    pub fn new() -> Self {
        Login {
            api_builder: ApiBuilder::new(),
            login: String::new(),
            password: String::new(),
            host: None,
        }
    }

    pub fn ask_auth(&mut self) -> &mut Login {
        println!("Please enter your username/email: ");
        let stdin = io::stdin();
        self.login = stdin.lock().lines().next().unwrap().unwrap();
        println!("Please enter your password: ");
        self.password = rpassword::read_password().unwrap();
        self
    }

    pub fn set_host(&mut self, host: Option<String>) -> &mut Login {
        self.host = host;
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        
        let url = match self.host.clone() {
            Some(h) => {
                let mut u = String::from("https://");
                u.push_str(&h);
                u.push_str("/ocs/v2.php/core/getapppassword");
                u
            },
            None => "/ocs/v2.php/core/getapppassword".to_owned(),
        };
        dbg!(url.clone());
        self.api_builder.set_url(Method::GET, &url);
        self.api_builder.set_header("OCS-APIRequest", HeaderValue::from_str("true").unwrap());
        self.api_builder.set_basic_auth(self.login.clone(), self.password.clone());
        self.api_builder.send().await
    }
    
    pub async fn send_with_err(&mut self) -> Result<String, ApiError> {
            match self.send().await {
                Err(res) => Err(ApiError::RequestError(res)),
                Ok(res) if res.status().is_success() => {
                    let body = res
                        .text()
                        .await
                        .map_err(|err| ApiError::EmptyError(err))?;
                    Ok(body)
                },
                Ok(res) => {
                    Err(ApiError::IncorrectRequest(res))
                }
            }
    }

    pub async fn send_login(&mut self) -> Result<String, ApiError> {
        match self.send_with_err().await {
            Ok(body) => Ok(self.parse(body)),
            Err(err) => Err(err),
        }
    }

    fn parse(&self, xml: String) -> String {
    let cursor = Cursor::new(xml);
        let parser = EventReader::new(cursor);

        let mut should_get = false;

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    should_get = {
                        if &name.local_name == "apppassword" {
                            true
                        } else {
                            false
                        }
                    };
                }
                Ok(XmlEvent::Characters(text)) => {
                    if !text.trim().is_empty() && should_get {
                        return text.clone();
                    }
                }
                Ok(XmlEvent::EndElement { name, .. }) => {
                }
                Err(e) => {
                    eprintln!("err: parsing xml: {}", e);
                    break;
                }
                _ => {}
            }
        }
        String::new()
    }
}
