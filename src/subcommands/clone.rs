use clap::{App, Arg, SubCommand, ArgMatches};
use std::borrow::Cow;
use textwrap::{fill, Options};

use crate::commands::clone::{self, CloneArgs};
use crate::global;
use crate::commands;

fn sized_str<'a>(content: &'a str) -> &'a str {
    fill(content, Options::new(70).width).as_str();
        "ok"
}

fn test() -> String {
    String::from("ok")
}

pub fn create() -> App<'static, 'static> {
    let remote_desc = sized_str(&format!("The repository to clone from. See the NEXTSYNC URLS section below for more information on specifying repositories."));
    let depth_desc = sized_str(&format!("Depth of the recursive fetch of object properties. This value should be lower when there are a lot of files per directory and higher when there are a lot of subdirectories with fewer files. (Default: {})", clone::DEPTH));
    SubCommand::with_name("clone")
        .arg(
            Arg::with_name("remote")
            .required(true)
            .takes_value(true)
            .value_name("REMOTE")
            //.help(_desc)
            )
        .arg(
            Arg::with_name("depth")
            .short("d")
            .long("depth")
            .required(false)
            .takes_value(true)
            //.help(&depth_desc)
            )
        .arg(
            Arg::with_name("directory")
            .required(false)
            .takes_value(true)
            .value_name("DIRECTORY")
            )
        .about("Clone a repository into a new directory")
        .after_help("NEXTSYNC URLS\nThe following syntaxes may be used:\n\t- user@host.xz/path/to/repo\n\t- http[s]://host.xz/apps/files/?dir=/path/to/repo&fileid=111111\n\t- [http[s]://]host.xz/remote.php/dav/files/user/path/to/repo\n")
}

pub fn handler(args: &ArgMatches<'_>) {
    if let Some(val) = args.values_of("directory") {
        global::global::set_dir_path(String::from(val.clone().next().unwrap()));
    }
    if let Some(remote) = args.values_of("remote") {
        commands::clone::clone(CloneArgs {
            remote,
            depth: args.values_of("depth").map(
                |mut val| val.next().unwrap().to_owned()
                ),
        });
    }
}
