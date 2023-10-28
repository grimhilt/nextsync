use clap::{App, Arg, SubCommand};

pub fn create() -> App<'static, 'static> {
    SubCommand::with_name("push")
        .about("Push changes on nextcloud")
}
