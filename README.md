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
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

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

      --no-read-min-edition
          Don't read the `edition` of the crate and do not use its value to reduce the search space

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
