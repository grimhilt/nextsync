use clap::{App, Arg, SubCommand};
mod commands;
mod utils;
mod services;
mod global;

fn main() {
    let matches = App::new("NextSync")
        .version("1.0")
        .author("grimhilt")
        .about("")
        .subcommand(
            SubCommand::with_name("init")
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            )
        .subcommand(
            SubCommand::with_name("status")
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            )
        .subcommand(SubCommand::with_name("reset"))
        .subcommand(SubCommand::with_name("push"))
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
            commands::add::add(files);
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
    } else if let Some(matches) = matches.subcommand_matches("push") {
        commands::push::push();
    } else if let Some(matches) = matches.subcommand_matches("config") {
        if let Some(mut var) = matches.values_of("variable") {
            if let Some(mut val) = matches.values_of("value") {
                commands::config::set(var.next().unwrap(), val.next().unwrap());
            }
        }
    }
}

