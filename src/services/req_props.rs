use std::io::Cursor;
use futures_util::StreamExt;
use chrono::{Utc, DateTime};
use reqwest::{Method, Response, Error};
use xml::reader::{EventReader, XmlEvent};
use reqwest::header::HeaderValue;
use crate::utils::time::parse_timestamp;
use crate::utils::api::{get_relative_s, ApiProps};
use crate::services::api::{ApiBuilder, ApiError};

#[derive(Debug)]
pub struct ObjProps {
    pub href: Option<String>,
    pub relative_s: Option<String>,
    pub lastmodified: Option<DateTime<Utc>>,
    pub contentlength: Option<u64>,
}

impl Clone for ObjProps {
    fn clone(&self) -> Self {
        ObjProps {
            href: self.href.clone(),
            relative_s: self.relative_s.clone(),
            lastmodified: self.lastmodified.clone(),
            contentlength: self.contentlength.clone(),
        }
    }
}

impl ObjProps {
    pub fn new() -> Self {
        ObjProps {
            href: None,
            relative_s: None,
            lastmodified: None,
            contentlength: None,
        }
    }
}

pub struct ReqProps {
    api_builder: ApiBuilder,
    xml_balises: Vec<String>,
    xml_payload: String,
    api_props: Option<ApiProps>
}

impl ReqProps {
    pub fn new() -> Self {
        ReqProps {
            api_builder: ApiBuilder::new(),
            xml_balises: vec![],
            xml_payload: String::new(),
            api_props: None,
        }
    }

    pub fn set_url(&mut self, url: &str) -> &mut ReqProps {
        self.api_builder.build_request(Method::from_bytes(b"PROPFIND").unwrap(), url);
        self
    }

    pub fn set_request(&mut self, p: &str, api_props: &ApiProps) -> &mut ReqProps {
        self.api_props = Some(api_props.clone());
        self.api_builder.set_req(Method::from_bytes(b"PROPFIND").unwrap(), p, api_props);
        self
    }

    pub fn gethref(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("href"));
        self
    }

    pub fn getlastmodified(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("getlastmodified"));
        self.xml_payload.push_str(r#"<d:getlastmodified/>"#);
        self
    }

    pub fn getcontentlength(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("getcontentlength"));
        self.xml_payload.push_str(r#"<d:getcontentlength/>"#);
        self
    }
    
    pub fn _getcontenttype(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("getcontenttype"));
        self.xml_payload.push_str(r#"<d:getcontenttype/>"#);
        self
    }

    pub fn _getpermissions(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("permissions"));
        self.xml_payload.push_str(r#"<oc:permissions/>"#);
        self
    }

    pub fn _getressourcetype(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("resourcetype"));
        self.xml_payload.push_str(r#"<d:resourcetype/>"#);
        self
    }

    pub fn _getetag(&mut self) -> &mut ReqProps {
        self.xml_balises.push(String::from("getetag"));
        self.xml_payload.push_str(r#"<d:getetag/>"#);
        self
    }

    pub fn set_depth(&mut self, depth: &str) -> &mut ReqProps {
        self.api_builder.set_header("Depth", HeaderValue::from_str(depth).unwrap());
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

    pub fn send_with_err(&mut self) -> Result<String, ApiError> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
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
        })
    }

    pub fn send_req_multiple(&mut self) -> Result<Vec<ObjProps>, ApiError> {
        match self.send_with_err() {
            Ok(body) => Ok(self.parse(body, true)),
            Err(err) => Err(err),
        }
    }

    pub fn send_req_single(&mut self) -> Result<ObjProps, ApiError> {
        // set depth to 0 as we only need one element
        self.set_depth("0");
        match self.send_with_err() {
            Ok(body) => {
                let objs = self.parse(body, false);
                let obj = objs[0].clone();
                Ok(obj)
            },
            Err(err) => Err(err),
        }
    }

    fn parse(&self, xml: String, multiple: bool) -> Vec<ObjProps> {
        let cursor = Cursor::new(xml);
        let parser = EventReader::new(cursor);

        let mut values: Vec<ObjProps> = vec![];

        let mut should_get = false;
        let mut val: String = String::from("");
        let mut content = ObjProps::new();

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    should_get = {
                        if self.xml_balises.clone().contains(&name.local_name) {
                            val = name.local_name.clone();
                            true
                        } else {
                            false
                        }
                    };
                }
                Ok(XmlEvent::Characters(text)) => {
                    if !text.trim().is_empty() && should_get {
                        match val.as_str() {
                            "href" => {
                                content.href = Some(text.clone());
                                content.relative_s = Some(
                                    get_relative_s(text, &(self.api_props
                                                           .clone()
                                                           .unwrap())));
                            },
                            "getlastmodified" => {
                                content.lastmodified = Some(
                                    parse_timestamp(&text).unwrap());
                            },
                            "getcontentlength" => {
                                content.contentlength = Some(
                                    text.clone().parse().unwrap());
                            },
                            _ => (),
                        }
                        should_get = false;
                    }
                }
                Ok(XmlEvent::EndElement { name, .. }) => {
                    if name.local_name == "response" {
                       values.push(content.clone());
                        if multiple {
                            content = ObjProps::new();
                        } else {
                            break;
                       }
                    }
                    should_get = false;
                }
                Err(e) => {
                    eprintln!("err: parsing xml: {}", e);
                    break;
                }
                _ => {}
            }
        }
        values
    }
}
