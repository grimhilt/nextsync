use reqwest::Client;
use std::fs::File;
use std::io::Read;
use reqwest::header::{HeaderValue, CONTENT_TYPE, HeaderMap};
use std::error::Error;
use std::env;
use dotenv::dotenv;

async fn send_propfind_request() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    
    let mut api_endpoint = env::var("HOST").unwrap().to_owned();
    api_endpoint.push_str("/remote.php/dav/files/");
    let username = env::var("USERNAME").unwrap();
    api_endpoint.push_str(&username);
    api_endpoint.push_str("/test");
    let password = env::var("PASSWORD").unwrap();

    // Create a reqwest client
    let client = Client::new();

    // Create the XML payload
    let xml_payload = r#"<?xml version="1.0" encoding="UTF-8"?>
        <d:propfind xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
          <d:prop>
            <d:getlastmodified/>
            <d:getcontentlength/>
            <d:getcontenttype/>
            <oc:permissions/>
            <d:resourcetype/>
            <d:getetag/>
          </d:prop>
        </d:propfind>"#;
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));


    // Send the request
    let response = client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), api_endpoint)
        .basic_auth(username, Some(password))
        .headers(headers)
        .body(xml_payload)
        .send()
        .await?;

    // Handle the response
    if response.status().is_success() {
        let body = response.text().await?;
        println!("Response body: {}", body);
    } else {
        println!("Request failed with status code: {}", response.status());
    }

    Ok(())
}

async fn upload_file(path: &str) -> Result<(), Box<dyn Error>> { 
    dotenv().ok();
    
    let mut api_endpoint = env::var("HOST").unwrap().to_owned();
    api_endpoint.push_str("/remote.php/dav/files/");
    let username = env::var("USERNAME").unwrap();
    api_endpoint.push_str(&username);
    api_endpoint.push_str("/test/ok");
    let password = env::var("PASSWORD").unwrap();
    
    let mut file = File::open("./file.test")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Create a reqwest client
    let client = Client::new();

    // Send the request
    let response = client
        .request(reqwest::Method::PUT, api_endpoint)
        .basic_auth(username, Some(password))
        .body(buffer)
        .send()
        .await?;

    // Handle the response
    if response.status().is_success() {
        let body = response.text().await?;
        println!("Response body: {}", body);
    } else {
        println!("Request failed with status code: {}", response.status());
    }
    Ok(())
}

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        if let Err(err) = upload_file("tkt").await {
            eprintln!("Error: {}", err);
        }
    });
}
