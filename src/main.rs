use clap::{App, Arg, SubCommand};
use crate::commands::add::AddArgs;

mod commands;
mod utils;
mod services;
mod global;
mod store;

fn main() {
    let matches = App::new("Nextsync")
        .version("1.0")
        .author("grimhilt")
        .about("A git-line command line tool to interact with nextcloud")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("clone")
            .arg(
                Arg::with_name("remote")
                .required(true)
                .takes_value(true)
                .value_name("REMOTE")
                )
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            .about("Clone a repository into a new directory")
            )
        .subcommand(
            SubCommand::with_name("init")
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            .about("Create an empty Nextsync repository") // Create an empty Git repository or reinitialize an existing one
            )
        .subcommand(
            SubCommand::with_name("status")
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            .about("Show the working tree status")
            )
        .subcommand(
            SubCommand::with_name("reset")
            .about("Clear the index")
            )
        .subcommand(
            SubCommand::with_name("push")
            .about("Push changes on nextcloud")
            )
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
            .arg(
                Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Allow adding otherwise ignored files."),
                )
            .about("Add changes to the index")
            )
        .subcommand(
            SubCommand::with_name("config")
            .arg(
                Arg::with_name("variable")
                .required(true)
                .takes_value(true)
                .value_name("VARIABLE")
                )
            .arg(
                Arg::with_name("value")
                .required(true)
                .takes_value(true)
                .value_name("VALUE")
                )
            )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        if let Some(val) = matches.values_of("directory") {
            global::global::set_dir_path(String::from(val.clone().next().unwrap()));
        }
        commands::init::init();
    } else if let Some(matches) = matches.subcommand_matches("status") {
        if let Some(val) = matches.values_of("directory") {
            global::global::set_dir_path(String::from(val.clone().next().unwrap()));
        }
        commands::status::status();
    } else if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(files) = matches.values_of("files") {
            commands::add::add(AddArgs {
                files: files,
                force: matches.is_present("force"),
            });
        }
    } else if let Some(_) = matches.subcommand_matches("reset") {
        commands::reset::reset();
    } else if let Some(matches) = matches.subcommand_matches("clone") {
        if let Some(val) = matches.values_of("directory") {
            global::global::set_dir_path(String::from(val.clone().next().unwrap()));
        }
        if let Some(remote) = matches.values_of("remote") {
            commands::clone::clone(remote);
        }
    } else if let Some(_matches) = matches.subcommand_matches("push") {
        commands::push::push();
    } else if let Some(matches) = matches.subcommand_matches("config") {
        if let Some(mut var) = matches.values_of("variable") {
            if let Some(mut val) = matches.values_of("value") {
                if commands::config::set(var.next().unwrap(), val.next().unwrap()).is_err() {
                    eprintln!("fatal: cannot save the value");
                }
            }
        }
    }
}

