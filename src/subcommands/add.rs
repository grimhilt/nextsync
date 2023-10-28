use clap::{App, Arg, SubCommand, ArgMatches};

use crate::commands;
use crate::commands::add::AddArgs;

pub fn create() -> App<'static, 'static> {
    SubCommand::with_name("add")
        .arg(
            Arg::with_name("files")
            .required(true)
            .conflicts_with("all")
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
        .arg(
            Arg::with_name("all")
            .short("A")
            .long("all")
            .help("This adds, modifies, and removes index entries to match the working tree"),
            )
        .about("Add changes to the index")
}

pub fn handler(args: &ArgMatches<'_>) {
    commands::add::add(AddArgs {
        files: args.values_of("files"),
        force: args.is_present("force"),
        all: args.is_present("all"),
    });
}
