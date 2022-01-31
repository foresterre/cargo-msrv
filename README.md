# cargo-msrv

This crate can assist you in finding the Minimum Supported Rust Version for a crate.

In this readme you'll find everything to get you started. You can find more detailed explanations in the
[cargo-msrv book](https://foresterre.github.io/cargo-msrv/index.html). 

### Install

With Cargo from crates.io [latest release]:

`cargo install cargo-msrv` to install or <br>
`cargo install cargo-msrv --force` to update

With Cargo from Github [latest development version]:

`cargo install cargo-msrv --git https://github.com/foresterre/cargo-msrv.git --branch main`

From the Arch Linux [community repository](https://archlinux.org/packages/community/x86_64/cargo-msrv/):

`pacman -S cargo-msrv`

### Preview

[![asciicast](https://asciinema.org/a/mFs1cjmjhCDinQNepooGelnYF.svg)](https://asciinema.org/a/mFs1cjmjhCDinQNepooGelnYF)

### Usage

* [`cargo msrv`](https://foresterre.github.io/cargo-msrv/commands/find.html) or [`cargo msrv --linear`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a Cargo project in your current working directory. 
* [`cargo msrv --path <dir>`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a Cargo project in the `<dir>` directory.
* [`cargo msrv -- <command>`](https://foresterre.github.io/cargo-msrv/commands/find.html) to use `<command>` as the compatibility check which decides whether a Rust version is
  compatible or not. This command should be runnable through rustup as `rustup run <toolchain> <command>`.
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
cargo-msrv
Martijn Gribnau <garm@ilumeo.com>
Helps with finding the Minimal Supported Rust Version (MSRV)

USAGE:
    cargo msrv [OPTIONS] or cargo-msrv [OPTIONS]

OPTIONS:
        --bisect
            Use a binary search to find the MSRV instead of a linear search

    -h, --help
            Prints help information

        --include-all-patch-releases
            Include all patch releases, instead of only the last

        --linear
            Use a linear search to find the MSRV, by checking toolchains from latest to earliest

        --ignore-lockfile
            Temporarily removes the lockfile, so it will not interfere with the building process. This is important when
            testing against Rust versions prior to 1.38.0, for which Cargo does not recognize the new v2 lockfile.
        --log-level <log_level>
            Specify the verbosity of logs the program should output [default: info]  [possible values:
            error, warn, info, debug, trace]
        --log-target <log_target>
            Specify where the program should output its logs [default: file]  [possible values: file,
            stdout]
        --max <max>
            Latest (most recent) version to take into account.Version must match a valid Rust toolchain, and be semver
            compatible. [aliases: maximum]
        --min <min>
            Earliest (least recent) version to take into account. Version must match a valid Rust toolchain, and be
            semver compatible. Edition aliases may also be used. [aliases: minimum]
        --no-check-feedback
            If provided, the outcome of each individual check will not be printed.

        --no-log
            Disable logging

        --no-read-min-edition
            If provided, the 'package.edition' value in the Cargo.toml will not be used to reduce search space.

        --output-format <output_format>
            Output status messages in machine-readable format. Machine-readable status updates will be printed in the
            requested format to stdout. [possible values: json, void]
        --release-source <release_source>
            Select the rust-releases source to use as the release index [default: rust-changelog]  [possible
            values: rust-changelog, rust-dist]
        --path <DIR>
            Path to the cargo project directory

        --target <TARGET>
            Check against a custom target (instead of the rustup default)

        --toolchain-file
            Output a rust-toolchain file with the MSRV as toolchain. The toolchain file will pin the Rust version for
            this crate. See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more.
    -V, --version
            Prints version information

        --verify
            Verify the MSRV defined in the 'package.rust-version' or the 'package.metadata.msrv' key in Cargo.toml. When
            this flag is present, cargo-msrv will not attempt to determine the true MSRV. Instead it attempts to verify
            whether for the specified MSRV, the `check` command passes. This is similar to how we determine whether a
            Rust toolchain version is compatible for your crate or not. DEPRECATED: use the `cargo msrv verify`
            subcommand instead.

ARGS:
    <COMMAND>...
            If given, this command is used to validate if a Rust version is compatible. Should be available to rustup,
            i.e. the command should work like so: `rustup run <toolchain> <COMMAND>`. The default check action is `cargo
            check --all`.

SUBCOMMANDS:
    help
            Prints this message or the help of the given subcommand(s)

    list
            List the MSRV's specified by dependency crate authors.

    show
            Show the MSRV of your crate, as specified in the Cargo manifest.

    verify
            Verify whether the MSRV is satisfiable. The MSRV must be specified using the 'package.rust-version' or
            'package.metadata.msrv' key in the Cargo.toml manifest.

An argument provided after two dashes (`--`), will be interpreted as a custom command `check` command, used to validate
whether a Rust toolchain version is compatible. The default `check` command is "cargo build". A custom `check` command
should be runnable by rustup, as they will be passed on to rustup like so: `rustup run <toolchain> <COMMAND...>`. You'll
only need to provide the <COMMAND...> part.
```

### JSON format

JSON output may be enabled by providing the `--output-format json` flag:

* determine msrv: `cargo msrv --output-format json`, or
* verify msrv: `cargo msrv --output-format json verify`, or
* list msrv's: `cargo msrv --output-format json list`

When the output format is 'json', various types of status messages can be printed. Each type is indicated
by the `reason` key.

#### Report mode


Reports the mode which will be used by `cargo-msrv`. These are the currently available modes:
* `determine-msrv`
* `verify-msrv`
* `list-msrv`

```jsonc
{
  "reason": "mode",
  // The mode in which cargo-msrv will operate
  "mode": "determine-msrv" /* OR */ "mode": "verify-msrv" /* OR */ "list-msrv" ,
   // The toolchain that will be used
  "toolchain":"x86_64-unknown-linux-gnu",
  // Command used to check a version. The key will be absent for mode 'list'
  "check_cmd":"cargo check --all"
}
```

#### Installing and Checking

Reported when a toolchain will be installed, or when a toolchain will
be run to check whether the version of the toolchain is compatible.

```jsonc
{
  "reason": "installing", /* OR */ "reason": "checking",
  // The current version being installed or checked
  "version": "1.25.0",
  // The number of versions checked before this
  "step": 0,
  // The total number of versions to be checked
  "total": 55,
  // The toolchain that is being used
  "toolchain": "x86_64-unknown-linux-gnu",
  // The command used to check each version. The key will be absent for mode 'list'
  "check_cmd": "cargo check --all"
}
```

#### Check complete

Reported when a check, which determines whether the toolchain version under test
is compatible, completes.

```jsonc
{
  "reason": "check-complete",
  // The version that was just checked
  "version": "1.25.0",
  // The number of versions checked before this
  "step": 0,
  // The total number of versions to be checked
  "total": 55,
  // true if this version is supported
  "success": false,
  // The toolchain that is being used
  "toolchain": "x86_64-unknown-linux-gnu",
  // The command used to check each version
  "check_cmd": "cargo check --all"
}
```

#### MSRV completed

Reported when all actions for a mode have been run to completion. 

```jsonc
{
  "reason": "msrv-complete" /* OR */ "reason": "verify-complete",
  // true if a msrv was found
  "success": true,
  // the msrv if found. The key will be absent if msrv wasn't found
  "msrv": "1.42.0",
  // The toolchain that is being used
  "toolchain": "x86_64-unknown-linux-gnu",
  // The command used to check each version. The key will be absent for mode 'list'
  "check_cmd": "cargo check --all"
}
```

#### List MSRV's specified by crate authors

Reported upon completion of listing the MSRV's of dependencies for a given crate.
The `list` output depends on the `variant`.
```jsonc
{
  "reason": "list",
  // output variant
  "variant": "ordered-by-msrv" /* OR */ "direct-deps",
  // always success when returning a result
  "success": true,
  // The output of the list subcommand
  "list": [
    /* when variant = 'ordered-by-msrv */
    {
        "msrv": "<msrv>",
        "dependencies": ["<dependencies which have this msrv>", ...]
    }
    /* OR, when variant = direct-deps */
    {
        "dependency": "<dependency crate name>",
        "version": "<dependency crate version>",
        "msrv": "<dependency crate msrv>",
        "depends_on": ["<dependencies of direct dependency crate>", ...]
    }
  ],
}
```

### Testing

Tests should be run with a single thread, because otherwise `rustup` uses the a single place for the download cache of a
specific toolchain version, and our multiple tests may attempt to overwrite or move the same cached version causing the
tests to get stuck and fail. You can achieve the above with the following Cargo command: `cargo test -- --test-threads=1`.

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
