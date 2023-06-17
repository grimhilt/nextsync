use crate::services::api::{ApiBuilder, ApiError};
use crate::utils::api::{ApiProps, get_relative_s};
use xml::reader::{EventReader, XmlEvent};
use std::io::Cursor;
use reqwest::{Method, Response, Error};

#[derive(Debug)]
pub struct FolderContent {
    pub href: Option<String>,
    pub relative_s: Option<String>,
}

impl Clone for FolderContent {
    fn clone(&self) -> Self {
        FolderContent {
            href: self.href.clone(),
            relative_s: self.relative_s.clone(),
        }
    }
}

impl FolderContent {
    fn new() -> Self {
        FolderContent {
            href: None,
            relative_s: None,
        }
    }
}

pub struct ListFolders {
    api_builder: ApiBuilder,
    xml_balises: Vec<String>,
    api_props: Option<ApiProps>
}

impl ListFolders {
    pub fn new() -> Self {
        ListFolders {
            api_builder: ApiBuilder::new(),
            xml_balises: vec![],
            api_props: None,
        }
    }

    pub fn set_url(&mut self, url: &str) -> &mut ListFolders {
        self.api_builder.build_request(Method::from_bytes(b"PROPFIND").unwrap(), url);
        self
    }

    pub fn set_request(&mut self, p: &str, api_props: &ApiProps) -> &mut ListFolders {
        self.api_props = Some(api_props.clone());
        self.api_builder.set_req(Method::from_bytes(b"PROPFIND").unwrap(), p, api_props);
        self
    }

    pub fn gethref(&mut self) -> &mut ListFolders {
        self.xml_balises.push(String::from("href"));
        self
    }

    pub async fn send(&mut self) -> Result<Response, Error> {
        self.api_builder.send().await
    }
    
    pub async fn send_with_err(&mut self) -> Result<String, ApiError> {
        let res = self.send().await.map_err(ApiError::RequestError)?; 
        if res.status().is_success() {
            let body = res.text().await.map_err(ApiError::EmptyError)?;
            Ok(body)
        } else {
            Err(ApiError::IncorrectRequest(res))
        }
    }

    pub async fn send_with_res(&mut self) -> Result<Vec<FolderContent>, ApiError> {
        match self.send_with_err().await {
            Ok(body) => Ok(self.parse(body)),
            Err(err) => Err(err),
        }
    }

    pub fn parse(&self, xml: String) -> Vec<FolderContent> {
        let cursor = Cursor::new(xml);
        let parser = EventReader::new(cursor);

        let mut should_get = false;
        let mut values: Vec<FolderContent> = vec![];

        let mut iter = self.xml_balises.iter();
        let mut val = iter.next();
        let mut content = FolderContent::new();

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    if let Some(v) = val.clone() {
                        should_get = &name.local_name == v;
                    } else {
                        // end of balises to get then start over for next object
                        values.push(content.clone());
                        iter = self.xml_balises.iter();
                        val = iter.next();
                        content = FolderContent::new();
                        if let Some(v) = val.clone() {
                            should_get = &name.local_name == v;
                        }
                    }
                }
                Ok(XmlEvent::Characters(text)) => {
                    if !text.trim().is_empty() && should_get {
                        match val.unwrap().as_str() {
                            "href" => {
                                content.href = Some(text.clone());
                                content.relative_s = Some(get_relative_s(text, &(self.api_props.clone().unwrap())));
                                dbg!(content.relative_s.clone());
                            },
                            _ => (),
                        }
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
