use clap::{App, Arg, SubCommand, ArgMatches};

use crate::global;
use crate::commands;

pub fn create() -> App<'static, 'static> {
    SubCommand::with_name("pull")
        .arg(
            Arg::with_name("path")
            .required(false)
            .takes_value(true)
            .value_name("PATH")
            .help("The path to pull."),
            )
        .about("Fetch and integrate changes from the nextcloud server.")
}

pub fn handler(args: &ArgMatches<'_>) {
    if let Some(val) = args.values_of("path") {
        global::global::set_dir_path(String::from(val.clone().next().unwrap()));
    }
    commands::pull::pull();
}
