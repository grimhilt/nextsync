use clap::{App, Arg, SubCommand, ArgMatches};

use crate::commands;

pub fn create() -> App<'static, 'static> {
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
}

pub fn handler(args: &ArgMatches<'_>) {
    if let Some(mut var) = args.values_of("variable") {
        if let Some(mut val) = args.values_of("value") {
            if commands::config::set(var.next().unwrap(), val.next().unwrap()).is_err() {
                eprintln!("fatal: cannot save the value");
            }
        }
    }
}
