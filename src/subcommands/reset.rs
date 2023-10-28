use clap::{App, Arg, SubCommand};

pub fn create() -> App<'static, 'static> {
    SubCommand::with_name("reset")
        .about("Clear the index")
}
