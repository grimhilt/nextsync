use clap::{App, Arg, SubCommand};
mod commands;
mod utils;
mod services;
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
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("reset"))
        .subcommand(
            SubCommand::with_name("clone")
            .arg(
                Arg::with_name("remote")
                .required(true)
                .takes_value(true)
                .value_name("REMOTE")
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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        match matches.values_of("directory") {
            Some(d) => commands::init::init(d.clone().next()),
            None => commands::init::init(None),
        }
    } else if let Some(_) = matches.subcommand_matches("status") {
        commands::status::status();
    } else if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(files) = matches.values_of("files") {
            commands::add::add(files);
        }
    } else if let Some(_) = matches.subcommand_matches("reset") {
        commands::reset::reset();
    } else if let Some(matches) = matches.subcommand_matches("clone") {
        if let Some(remote) = matches.values_of("remote") {
            commands::clone::clone(remote);
        }
    }
}
