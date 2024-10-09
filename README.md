# cargo-msrv

This crate can assist you in finding the Minimum Supported Rust Version for a crate.

In this readme you'll find everything to get you started. You can find more detailed explanations in the
[cargo-msrv book](https://foresterre.github.io/cargo-msrv/index.html).

### Install

#### cargo ([crates.io source](https://crates.io/crates/cargo-msrv))

| cargo       | supported | command                               |
|-------------|-----------|---------------------------------------|
| stable      | 💚        | `$ cargo install cargo-msrv --locked` |
| development | ❌         |                                       |

#### cargo ([git source](https://github.com/foresterre/cargo-msrv))

| cargo       | supported | command                                                                                       |
|-------------|-----------|-----------------------------------------------------------------------------------------------|
| stable      | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git --tag v0.16.0` cargo-msrv |
| development | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git` cargo-msrv               |

#### [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

| cargo       | supported | command                                                     |
|-------------|-----------|-------------------------------------------------------------|
| stable      | 💚        | `$ cargo binstall --version 0.16.0 --no-confirm cargo-msrv` |
| development | ❌         |                                                             |

#### Arch Linux [extra repository](https://archlinux.org/packages/extra/x86_64/cargo-msrv/)

* `pacman -S cargo-msrv`

#### Docker

You can use the following commands for building and running via Docker:

* `$ docker build -t cargo-msrv .`
* `$ docker run -t -v "$(pwd)/Cargo.toml":/app/Cargo.toml cargo-msrv`

Docker images are also available on [Docker Hub](https://hub.docker.com/r/foresterre/cargo-msrv).

### Prerequisites

[Rustup](https://rustup.rs/) is required for the `cargo msrv (find)` and `cargo msrv verify` commands.

### Preview

[![asciicast](https://asciinema.org/a/679852.svg)](https://asciinema.org/a/679852)

### Usage

* [`cargo msrv find`](https://foresterre.github.io/cargo-msrv/commands/find.html)
  or [`cargo msrv find --linear`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a
  Cargo project in your current working directory.
* [`cargo msrv --path <dir> find`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a
  Cargo project in the `<dir>` directory.
* [`cargo msrv find -- <command>`](https://foresterre.github.io/cargo-msrv/commands/find.html) to use `<command>` as the
  compatibility check which decides whether a Rust version is
  compatible or not. This command should be runnable through rustup as `rustup run <toolchain> <command>`.
    * Example: `cargo msrv find -- cargo check --tests`.
* [`cargo msrv verify`](https://foresterre.github.io/cargo-msrv/commands/verify.html)  to verify the MSRV as specified
  by a crate author\
    * A crate author may specify the MSRV using the `package.rust-version` (Rust >=1.56) or the `package.metadata.msrv`
      key in the 'Cargo.toml' manifest. See
      the [book](https://foresterre.github.io/cargo-msrv/commands/list.html#description) for a more detailed
      description.
* [`cargo msrv list`](https://foresterre.github.io/cargo-msrv/commands/list.html) to list the MSRV's of your
  dependencies as specified by their authors
* [`cargo msrv show`](https://foresterre.github.io/cargo-msrv/commands/show.html) to show the currently specified MSRV

Please refer to the [commands](https://foresterre.github.io/cargo-msrv/commands/index.html) chapter in the cargo-msrv
book for more detailed descriptions of the supported (sub) commands.

**Options**

```
Find your Minimum Supported Rust Version!

Usage: cargo msrv [OPTIONS] <COMMAND>

Commands:
  find    Find the MSRV
  list    Display the MSRV's of dependencies
  set     Set the MSRV of the current crate to a given Rust version
  show    Show the MSRV of your crate, as specified in the Cargo manifest
  verify  Verify whether the MSRV is satisfiable
  help    Print this message or the help of the given subcommand(s)

Options:
      --path <Crate Directory>
          Path to cargo project directory

      --manifest-path <Cargo Manifest>
          Path to cargo manifest file

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

User output options:
      --output-format <FORMAT>
          Set the format of user output

          [default: human]

          Possible values:
          - human:   Progress bar rendered to stderr
          - json:    Json status updates printed to stdout
          - minimal: Minimal output, usually just the result, such as the MSRV or whether verify succeeded or failed

      --no-user-output
          Disable user output

Debug output options:
      --no-log
          Disable logging

      --log-target <LOG TARGET>
          Specify where the program should output its logs

          [default: file]
          [possible values: file, stdout]

      --log-level <LEVEL>
          Specify the severity of logs which should be

          [default: info]
          [possible values: trace, debug, info, warn, error]


            You may provide a custom compatibility `check` command as the last argument (only
            when this argument is provided via the double dash syntax, e.g. `$ cargo msrv -- custom
            command`.
            This custom check command will then be used to validate whether a Rust version is
            compatible.
            A custom `check` command should be runnable by rustup, as they will be passed on to
            rustup like so: `rustup run <toolchain> <COMMAND...>`. NB: You only need to provide the
            <COMMAND...> part.

            By default, the custom check command is `cargo check`.
```

### JSON format

JSON output may be enabled by providing the `--output-format json` flag: `cargo msrv find --output-format json`.
Events are printed as json lines. The event type is indicated by the `type` key.

Please see the [Output formats](https://foresterre.github.io/cargo-msrv/output-formats/index.html) and
[Output format: JSON](https://foresterre.github.io/cargo-msrv/output-formats/json.html) chapters of the
book for documentation of this output format.

### License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
