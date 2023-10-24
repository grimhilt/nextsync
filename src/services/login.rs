use std::io;
use std::io::Cursor;
use std::io::prelude::*;
use xml::reader::{EventReader, XmlEvent};
use reqwest::{header::HeaderValue, Method};
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
    fn new() -> Self {
        Login {
            api_builder: ApiBuilder::new(),
            login: String::new(),
            password: String::new(),
            host: None,
        }
    }

    fn send(&mut self) -> Result<Option<String>, ApiError> {
        
        let url = match self.host.clone() {
            Some(h) => {
                let mut u = String::from("https://");
                u.push_str(&h);
                u.push_str("/ocs/v2.php/core/getapppassword");
                u
            },
            None => "/ocs/v2.php/core/getapppassword".to_owned(),
        };
        self.api_builder.set_url(Method::GET, &url);
        self.api_builder.set_header("OCS-APIRequest", HeaderValue::from_str("true").unwrap());
        self.api_builder.set_basic_auth(self.login.clone(), self.password.clone());
        self.api_builder.send(true)
    }
}

impl Login {
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
    
    pub fn send_login(&mut self) -> Result<String, ApiError> {
        match self.send() {
            Ok(Some(body)) => Ok(self.parse(body)),
            Ok(None) => Err(ApiError::Unexpected(String::from("Empty after tested"))),
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
                //Ok(XmlEvent::EndElement { name, .. }) => {
                //}
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
