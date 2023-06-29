# NextSync

A git-like command line tool to interact with Nextcloud.

This is **work in progress**.

This should work pretty much like git with some adaptations to be more debuggable (for now) and easier to code. There is no history and with that no need to commit, to upload new files you have to add and push them.

## Features

- [x] Cloning
- [x] Status (only for new and deleted files/folders)
- [x] Pushing updates (only deletion and addition no changes)
- [x] Using a .nextsyncignore to ignore files
- [ ] Pulling changes
- [ ] Auth without using env variables
- [ ] Detecting local changes

## Usage

For the authentification, I use env variables (USERNAME and PASSWORD), this is temporary.

```
USAGE:
    nextsync [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add       Add changes to the index
    clone     Clone a repository into a new directory
    config
    help      Prints this message or the help of the given subcommand(s)
    init      Create an empty Nextsync repository
    push      Push changes on nextcloud
    reset     Clear the index
    status    Show the working tree status
```
