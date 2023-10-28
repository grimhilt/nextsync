use clap::{App, SubCommand};

mod subcommands;

mod commands;
mod utils;
mod services;
mod global;
mod store;

fn main() {
    let app = App::new("Nextsync")
        .version("1.0")
        .author("grimhilt")
        .about("A git-line command line tool to interact with nextcloud")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(subcommands::clone::create())
        .subcommand(subcommands::init::create())
        .subcommand(subcommands::status::create())
        .subcommand(subcommands::add::create())
        .subcommand(subcommands::push::create())
        .subcommand(subcommands::reset::create())
        .subcommand(subcommands::config::create())
        .subcommand(subcommands::remote_diff::create())
        .subcommand(subcommands::pull::create())
        .subcommand(
            SubCommand::with_name("test")
            );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("init", Some(args)) => subcommands::init::handler(args),
        ("status", Some(args)) => subcommands::status::handler(args),
        ("add", Some(args)) => subcommands::add::handler(args),
        ("reset", Some(_)) => commands::reset::reset(),
        ("clone", Some(args)) => subcommands::clone::handler(args),
        ("push", Some(_)) => commands::push::push(),
        ("config", Some(args)) => subcommands::config::handler(args),
        ("remote-diff", Some(args)) => subcommands::remote_diff::handler(args),
        ("pull", Some(args)) => subcommands::pull::handler(args),

        (_, _) => {},
    };
}

