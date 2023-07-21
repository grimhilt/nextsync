use clap::{App, Arg, SubCommand};
use textwrap::{fill, Options};

use crate::commands::add::AddArgs;
use crate::commands::remote_diff::RemoteDiffArgs;
use crate::commands::clone::{self, CloneArgs};

mod commands;
mod utils;
mod services;
mod global;
mod store;

fn main() {
    let matches = App::new("Nextsync")
        .version("1.0")
        .author("grimhilt")
        .about("A git-line command line tool to interact with nextcloud")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("clone")
            .arg(
                Arg::with_name("remote")
                .required(true)
                .takes_value(true)
                .value_name("REMOTE")
                .help(&fill(
                        "The repository to clone from. See the NEXTSYNC URLS section below for more information on specifying repositories.",
                        Options::new(70).width,
                        ))
                )
            .arg(
                Arg::with_name("depth")
                .short("d")
                .long("depth")
                .required(false)
                .takes_value(true)
                .help(&fill(
                        &format!("Depth of the recursive fetch of object properties. This value should be lower when there are a lot of files per directory and higher when there are a lot of subdirectories with fewer files. (Default: {})", clone::DEPTH),
                        Options::new(70).width,
                        ))
                )
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            .about("Clone a repository into a new directory")
            .after_help("NEXTSYNC URLS\nThe following syntaxes may be used:\n\t- user@host.xz/path/to/repo\n\t- http[s]://host.xz/apps/files/?dir=/path/to/repo&fileid=111111\n\t- [http[s]://]host.xz/remote.php/dav/files/user/path/to/repo\n")
            )
        .subcommand(
            SubCommand::with_name("init")
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            .about("Create an empty Nextsync repository") // Create an empty Git repository or reinitialize an existing one
            )
        .subcommand(
            SubCommand::with_name("status")
            .arg(
                Arg::with_name("directory")
                .required(false)
                .takes_value(true)
                .value_name("DIRECTORY")
                )
            .about("Show the working tree status")
            )
        .subcommand(
            SubCommand::with_name("reset")
            .about("Clear the index")
            )
        .subcommand(
            SubCommand::with_name("push")
            .about("Push changes on nextcloud")
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
            .arg(
                Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Allow adding otherwise ignored files."),
                )
            .about("Add changes to the index")
            )
        .subcommand(
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
            )
        .subcommand(
            SubCommand::with_name("remote-diff")
            .arg(
                Arg::with_name("path")
                .required(false)
                .takes_value(true)
                .value_name("PATH")
                .help("The path to pull."),
                )
            .about("Fetch new and modifed files from the nextcloud server.")
            )
        .subcommand(
            SubCommand::with_name("test")
            )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        if let Some(val) = matches.values_of("directory") {
            global::global::set_dir_path(String::from(val.clone().next().unwrap()));
        }
        commands::init::init();
    } else if let Some(matches) = matches.subcommand_matches("status") {
        if let Some(val) = matches.values_of("directory") {
            global::global::set_dir_path(String::from(val.clone().next().unwrap()));
        }
        commands::status::status();
    } else if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(files) = matches.values_of("files") {
            commands::add::add(AddArgs {
                files,
                force: matches.is_present("force"),
            });
        }
    } else if let Some(_) = matches.subcommand_matches("reset") {
        commands::reset::reset();
    } else if let Some(matches) = matches.subcommand_matches("clone") {
        if let Some(val) = matches.values_of("directory") {
            global::global::set_dir_path(String::from(val.clone().next().unwrap()));
        }
        if let Some(remote) = matches.values_of("remote") {
            commands::clone::clone(CloneArgs {
                remote,
                depth: matches.values_of("depth").map(
                    |mut val| val.next().unwrap().to_owned()
                    ),
            });
        }
    } else if let Some(_matches) = matches.subcommand_matches("push") {
        commands::push::push();
    } else if let Some(matches) = matches.subcommand_matches("config") {
        if let Some(mut var) = matches.values_of("variable") {
            if let Some(mut val) = matches.values_of("value") {
                if commands::config::set(var.next().unwrap(), val.next().unwrap()).is_err() {
                    eprintln!("fatal: cannot save the value");
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("remote-diff") {
        commands::remote_diff::remote_diff(RemoteDiffArgs {
            path: {
                if let Some(mut path) = matches.values_of("path") {
                    match path.next() {
                        Some(p) => Some(String::from(p)),
                        None => None,
                    }
                } else {
                    None
                }
            },
        });
    } else if let Some(_) = matches.subcommand_matches("test") {

    }
}

