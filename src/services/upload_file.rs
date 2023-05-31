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
