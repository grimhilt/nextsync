//use reqwest::Client;
//use std::fs::File;
//use std::io::Read;
//use reqwest::header::{HeaderValue, CONTENT_TYPE, HeaderMap};
//use std::error::Error;
//use std::env;
//use dotenv::dotenv;

use clap::{App, Arg, SubCommand};
mod commands;
mod utils;
fn main() {
    let matches = App::new("NextSync")
        .version("1.0")
        .author("grimhilt")
        .about("")
        .subcommand(SubCommand::with_name("init"))
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("reset"))
        .subcommand(
            SubCommand::with_name("add")
            .arg(
                Arg::with_name("files")
                .required(true)
                .multiple(true)
                .takes_value(true)
                .value_name("FILE")
                .help("Files to add"),
                )
            )
        .get_matches();

    if let Some(_) = matches.subcommand_matches("init") {
        commands::init::init();
    } else if let Some(_) = matches.subcommand_matches("status") {
        commands::status::status();
    } else if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(files) = matches.values_of("files") {
            commands::add::add(files);
        }
    } else if let Some(_) = matches.subcommand_matches("reset") {
        commands::reset::reset();
    }


    //tokio::runtime::Runtime::new().unwrap().block_on(async {
    //    if let Err(err) = upload_file("tkt").await {
    //        eprintln!("Error: {}", err);
    //    }
    //});
}
