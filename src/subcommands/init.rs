use clap::{App, Arg, SubCommand, ArgMatches};

use crate::global;
use crate::commands;

pub fn create() -> App<'static, 'static> {
    SubCommand::with_name("init")
        .arg(
            Arg::with_name("directory")
            .required(false)
            .takes_value(true)
            .value_name("DIRECTORY")
            )
        .about("Create an empty Nextsync repository")
        // Create an empty nextsync repository or reinitialize an existing one
}

pub fn handler(args: &ArgMatches<'_>) {
    if let Some(val) = args.values_of("directory") {
        global::global::set_dir_path(String::from(val.clone().next().unwrap()));
    }
    commands::init::init();
}
