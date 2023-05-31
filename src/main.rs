//use reqwest::Client;
//use std::fs::File;
//use std::io::Read;
//use reqwest::header::{HeaderValue, CONTENT_TYPE, HeaderMap};
//use std::error::Error;
//use std::env;
//use dotenv::dotenv;

use clap::{App, SubCommand};
mod commands;
fn main() {
    let matches = App::new("NextSync")
        .version("1.0")
        .author("grimhilt")
        .about("")
        .subcommand(SubCommand::with_name("init"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("init") {
        commands::init::init();
    }

    //tokio::runtime::Runtime::new().unwrap().block_on(async {
    //    if let Err(err) = upload_file("tkt").await {
    //        eprintln!("Error: {}", err);
    //    }
    //});
}
