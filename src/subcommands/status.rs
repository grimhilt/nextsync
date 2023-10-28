use clap::{App, Arg, SubCommand, ArgMatches};

use crate::global;
use crate::commands;
use crate::commands::status::StatusArgs;

pub fn create() -> App<'static, 'static> {
    SubCommand::with_name("status")
        .arg(
            Arg::with_name("directory")
            .required(false)
            .takes_value(true)
            .value_name("DIRECTORY")
            )
        .arg(
            Arg::with_name("nostyle")
            .long("nostyle")
            .help("Status with minium information and style"),
            )
        .about("Show the working tree status")
}

pub fn handler(args: &ArgMatches<'_>) {
    if let Some(val) = args.values_of("directory") {
        global::global::set_dir_path(String::from(val.clone().next().unwrap()));
    }
    commands::status::status(StatusArgs {
        nostyle: args.is_present("nostyle"),
    });
}
