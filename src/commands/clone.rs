use clap::Values;
use std::fs::{self, DirBuilder};
use std::path::Path;
use crate::services::list_folders::ListFolders;
use std::error::Error;

pub fn clone(remote: Values<'_>) {
    let url = remote.clone().next().unwrap();
    let path = Path::new(url);
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        call(url).await
    });
    //DirBuilder::new()
    //    .create(path.parent());
}

pub async fn call(url: &str) -> Result<String, Box<dyn Error>> {
    let response = ListFolders::new(url).send().await?;
    if response.status().is_success() {
        let body = response.text().await?;
        println!("Response body: {}", body);
    } else {
        println!("Request failed with status code: {}", response.status());
    }
    Ok(())
}

