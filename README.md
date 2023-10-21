# Nextsync

A git-like command line tool to interact with Nextcloud.

This is **in working progress**.

This should work pretty much like git with some adaptations to be more debuggable (for now) and easier to code. There is no history and with that no need to commit, to upload new files you have to add and push them.

## Features

- [x] Cloning
- [x] Status (new, deleted, modified, copied, moved)
- [x] Pushing updates (new, deleted, modified)
- [x] Using a .nextsyncignore to ignore files
- [ ] Pulling changes
- [x] Auth with a token
- [ ] Remember token
- [ ] Various optimisation

## Usage

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
