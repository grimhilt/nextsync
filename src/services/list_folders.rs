use crate::services::api::{ApiBuilder, ApiError};
use xml::reader::{EventReader, XmlEvent};
use std::io::Cursor;
use reqwest::{Method, IntoUrl, Response, Error};

pub struct FolderContent {
    pub href: Option<String>,
}

impl Clone for FolderContent {
    fn clone(&self) -> Self {
        FolderContent {
            href: self.href.clone(),
        }
    }
}

impl FolderContent {
    fn new() -> Self {
        FolderContent {
            href: None,
        }
    }
}

pub struct ListFolders {
    api_builder: ApiBuilder,
    xml_balises: Vec<String>,
}

impl ListFolders {
    pub fn new<U: IntoUrl>(url: U) -> Self {
        ListFolders {
            api_builder: ApiBuilder::new()
                .set_request(Method::from_bytes(b"PROPFIND").unwrap(), url),
            xml_balises: vec![],
        }
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
                            "href" => content.href = Some(text),
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
