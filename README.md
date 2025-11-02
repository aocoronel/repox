# A git repository utility

> [!WARNING]
> This repository has been moved to Codeberg. Find it at [aocoronel/repox](https://codeberg.org/aocoronel/repox).

`repox` is a tool that allows you to run git commands in a different local repositories, including cloning, fetching, pulling, pushing and checking status in bulk. Of course, everything concurrently.

Currently only the commands listed above are supported.

This project has two implementations: One in **Bash**, and the other in **Rust**. Both version work the same.

## Requirements

- git

## Installation

### Shell Version

```bash
git clone https://github.com/aocoronel/repox
cd repox
chmod +x src/repox && mv src/repox $HOME/.local/bin
```

### Rust Version

```bash
git clone https://github.com/aocoronel/repox
cd repox
cargo build --release
mv target/release/repox $HOME/.local/bin
```

## Usage

```
A git repository utility

Usage:
  repox FLAG <FLAG_INPUT> COMMAND SUB_DIRECTORY
  repox COMMAND github
  repox COMMAND codeberg

Commands:
  clone               Clone all repos
  fetch               Fetch all repos
  pull                Pull all repos
  status              Check status from all repos

Flags:
  -h, --help           Displays this message and exits
  -p <PARALLEL>        Set parallels to use
  -c <FILE>            Use a specific repox file
```

### Configuration

By default, `repox` looks for a file called `.repox` at the home directory, which can be changed using the `-c` flag.

The repox file should contain a link to a Git Remote per line, as follows:

```
ssh://git@github.com/aocoronel/doppel.git
ssh://git@github.com/aocoronel/dotfiles.git
https://github.com/aocoronel/repox.git
# https://github.com/aocoronel/repox.git # Comments are ignored
```

When running `repox` it will also use the `DEV` environment variable, which is the main directory where the git repositories are managed by `repox`.

### Examples

Base usage repox:

```bash
repox clone github
# Looks at .repox file at $HOME
# Performs clone operations inside $DEV/github
```

Change how many jobs are ran:

```bash
repox -p 10 clone github
# Looks at .repox file at $HOME
# Performs 10 clone operations simultaneously inside $DEV/github
# By default, parallels is set to 5
```

Read repox file from a different location:

```bash
repox -c dev/config/codeberg.txt clone codeberg
# Looks at dev/config/codeberg.txt file
# Performs clone operations inside $DEV/codeberg
```

## License

This repository is licensed under the MIT License, allowing for extensive use, modification, copying, and distribution.
