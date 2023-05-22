# cargo-msrv

This crate can assist you in finding the Minimum Supported Rust Version for a crate.

In this readme you'll find everything to get you started. You can find more detailed explanations in the
[cargo-msrv book](https://foresterre.github.io/cargo-msrv/index.html).

### Install

#### cargo ([crates.io source](https://crates.io/crates/cargo-msrv))

| cargo       | supported | command                                               |
|-------------|-----------|-------------------------------------------------------|
| stable      | 💚        | `$ cargo install cargo-msrv`                          |
| beta        | 💚        | `$ cargo install cargo-msrv --version 0.16.0-beta.14` |
| development | ❌        |                                                       |

#### cargo ([git source](https://github.com/foresterre/cargo-msrv))

| cargo       | supported | command                                                                                               |
|-------------|-----------|-------------------------------------------------------------------------------------------------------|
| stable      | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git --tag v0.15.1` cargo-msrv         |
| beta        | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git --tag v0.16.0-beta.14` cargo-msrv |
| development | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git` cargo-msrv                       |

#### [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

| cargo       | supported | command                                                             |
|-------------|-----------|---------------------------------------------------------------------|
| stable      | 💚        | `$ cargo binstall --version 0.15.1 --no-confirm cargo-msrv`         |
| beta        | 💚        | `$ cargo binstall --version 0.16.0-beta.13 --no-confirm cargo-msrv` |
| development | ❌        |                                                                     |

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

[![asciicast](https://asciinema.org/a/mFs1cjmjhCDinQNepooGelnYF.svg)](https://asciinema.org/a/mFs1cjmjhCDinQNepooGelnYF)

### Usage

* [`cargo msrv`](https://foresterre.github.io/cargo-msrv/commands/find.html) or [`cargo msrv --linear`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a Cargo project in your current working directory.
* [`cargo msrv --path <dir>`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a Cargo project in the `<dir>` directory.
* [`cargo msrv -- <command>`](https://foresterre.github.io/cargo-msrv/commands/find.html) to use `<command>` as the compatibility check which decides whether a Rust version is
  compatible or not. This command should be runnable through rustup as `rustup run <toolchain> <command>`.
  * Example: `cargo msrv -- cargo check --tests`.
* [`cargo msrv verify`](https://foresterre.github.io/cargo-msrv/commands/find.html)  to verify the MSRV as specified by a crate author\
  * A crate author may specify the MSRV using the `package.rust-version` (Rust >=1.56) or the `package.metadata.msrv` key
    in the 'Cargo.toml' manifest. See the [book](https://foresterre.github.io/cargo-msrv/commands/list.html#description)
    for a more detailed description.
* [`cargo msrv list`](https://foresterre.github.io/cargo-msrv/commands/list.html) to list the MSRV's of your dependencies as specified by their authors
* [`cargo msrv show`](https://foresterre.github.io/cargo-msrv/commands/show.html) to show the currently specified MSRV

Please refer to the [commands](https://foresterre.github.io/cargo-msrv/commands/index.html) chapter in the cargo-msrv
book for more detailed descriptions of the supported (sub) commands.


**Options**
```
Find your Minimum Supported Rust Version!

Usage: cargo msrv [OPTIONS] [-- <CUSTOM_CHECK_COMMAND>...] [COMMAND]

Commands:
  list
          Display the MSRV's of dependencies
  set
          Set the MSRV of the current crate to a given Rust version
  show
          Show the MSRV of your crate, as specified in the Cargo manifest
  verify
          Verify whether the MSRV is satisfiable. The MSRV must be specified using the 'package.rust-version' or 'package.metadata.msrv' key in the Cargo.toml manifest
  help
          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Find MSRV options:
      --bisect
          Use a binary search to find the MSRV (default)

          When the search space is sufficiently large, which is common, this is much faster than a linear search. A binary search will approximately halve the search space for each Rust version checked for compatibility.

      --linear
          Use a linear search to find the MSRV

          This method checks toolchain from the most recent release to the earliest.

      --write-toolchain-file
          Pin the MSRV by writing the version to a rust-toolchain file

          The toolchain file will pin the Rust version for this crate. See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more.

      --ignore-lockfile
          Temporarily remove the lockfile, so it will not interfere with the building process

          This is important when testing against older Rust versions such as Cargo versions prior to Rust 1.38.0, for which Cargo does not recognize the newer lockfile formats.

      --no-check-feedback
          Don't print the result of compatibility checks

          The feedback of a compatibility check can be useful to determine why a certain Rust version is not compatible. Rust usually prints very detailed error messages. While most often very useful, in some cases they may be too noisy or lengthy. If this flag is given, the result messages will not be printed.

      --write-msrv
          Write the MSRV to the Cargo manifest

          For toolchains which include a Cargo version which supports the rust-version field, the `package.rust-version` field will be written. For older Rust toolchains, the `package.metadata.msrv` field will be written instead.

Rust releases options:
      --min <VERSION_SPEC or EDITION>
          Least recent version or edition to take into account

          Given version must match a valid Rust toolchain, and be semver compatible, be a two component `major.minor` version. or match a Rust edition alias.

          For example, the edition alias "2018" would match Rust version `1.31.0`, since that's the first version which added support for the Rust 2018 edition.

      --max <VERSION_SPEC>
          Most recent version to take into account

          Given version must match a valid Rust toolchain, and be semver compatible, or be a two component `major.minor` version.

      --include-all-patch-releases
          Include all patch releases, instead of only the last

      --release-source <SOURCE>
          [default: rust-changelog]
          [possible values: rust-changelog, rust-dist]

Toolchain options:
      --target <TARGET>
          Check against a custom target (instead of the rustup default)

Custom check options:
      --path <Crate Directory>
          Path to cargo project directory

      --manifest-path <Cargo Manifest>
          Path to cargo manifest file

  [CUSTOM_CHECK_COMMAND]...
          Supply a custom `check` command to be used by cargo msrv

User output options:
      --output-format <FORMAT>
          Set the format of user output

          [default: human]

          Possible values:
          - human:
            Progress bar rendered to stderr
          - json:
            Json status updates printed to stdout
          - minimal:
            Minimal output, usually just the result, such as the MSRV or whether verify succeeded or failed

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

JSON output may be enabled by providing the `--output-format json` flag: `cargo msrv --output-format json`.
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
