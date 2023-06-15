use crate::services::api::{ApiBuilder, ApiError};
use xml::reader::{EventReader, XmlEvent};
use std::io::{self, Cursor};
use reqwest::{Method, IntoUrl, Response, Error};

pub struct ReqProps {
    api_builder: ApiBuilder,
    xml_list: Vec<String>,
    xml_payload: String,
}

impl ReqProps {
    pub fn new() -> Self {
        ReqProps {
            api_builder: ApiBuilder::new(),
            xml_list: vec![],
            xml_payload: String::new(),
        }
    }

    pub fn set_url(&mut self, url: &str) -> &mut ReqProps {
        self.api_builder.build_request(Method::from_bytes(b"PROPFIND").unwrap(), url);
        self
    }

    pub fn getlastmodified(&mut self) -> &mut ReqProps {
        self.xml_list.push(String::from("getlastmodified"));
        self.xml_payload.push_str(r#"<d:getlastmodified/>"#);
        self
    }

    pub fn getcontentlenght(&mut self) -> &mut ReqProps {
        self.xml_list.push(String::from("getcontentlength"));
        self.xml_payload.push_str(r#"<d:getcontentlength/>"#);
        self
    }
    
    pub fn getcontenttype(&mut self) -> &mut ReqProps {
        self.xml_list.push(String::from("getcontenttype"));
        self.xml_payload.push_str(r#"<d:getcontenttype/>"#);
        self
    }

    pub fn getpermissions(&mut self) -> &mut ReqProps {
        self.xml_list.push(String::from("permissions"));
        self.xml_payload.push_str(r#"<oc:permissions/>"#);
        self
    }

    pub fn getressourcetype(&mut self) -> &mut ReqProps {
        self.xml_list.push(String::from("resourcetype"));
        self.xml_payload.push_str(r#"<d:resourcetype/>"#);
        self
    }

    pub fn getetag(&mut self) -> &mut ReqProps {
        self.xml_list.push(String::from("getetag"));
        self.xml_payload.push_str(r#"<d:getetag/>"#);
        self
    }

    fn validate_xml(&mut self) -> &mut ReqProps {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><d:propfind xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns"><d:prop>"#);
        xml.push_str(&self.xml_payload.clone());
        xml.push_str(r#"</d:prop></d:propfind>"#);
        self.api_builder.set_xml(xml);
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.validate_xml();
        self.api_builder.send().await
    }

    pub async fn send_with_err(&mut self) -> Result<Vec<String>, ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.text().await.map_err(ApiError::EmptyError)?;
            Ok(self.parse(body))
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }

    pub fn parse(&self, xml: String) -> Vec<String> {
        let cursor = Cursor::new(xml);
        let parser = EventReader::new(cursor);

        let mut should_get = false;
        let mut values: Vec<String> = vec![];
        let mut iter = self.xml_list.iter();
        let mut val = iter.next();

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    if let Some(v) = val.clone() {
                        should_get = &name.local_name == v;
                    } else {
                        break;
                    }
                }
                Ok(XmlEvent::Characters(text)) => {
                    if !text.trim().is_empty() && should_get {
                        values.push(text);
                        val = iter.next()
                    }
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    should_get = false;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        values
    }
}
