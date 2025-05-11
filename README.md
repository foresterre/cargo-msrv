# cargo-msrv

This crate can assist you in finding the Minimum Supported Rust Version for a crate.

In this readme you'll find everything to get you started. You can find more detailed explanations in the
[cargo-msrv book](https://foresterre.github.io/cargo-msrv/index.html).

### Preview

<sup><strong>⌘</strong> [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

[![asciicast](https://asciinema.org/a/679852.svg)](https://asciinema.org/a/679852)

### Prerequisites

<sup>⌘ [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

[Rustup](https://rustup.rs/) is required for the `cargo msrv (find)` and `cargo msrv verify` commands.

### Install

<sup>⌘ [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

#### cargo ([crates.io source](https://crates.io/crates/cargo-msrv))

| cargo       | supported | command                               |
|-------------|-----------|---------------------------------------|
| stable      | 💚        | `$ cargo install cargo-msrv --locked` |
| development | ❌        |                                       |

#### cargo ([git source](https://github.com/foresterre/cargo-msrv))

| cargo       | supported | command                                                                                       |
|-------------|-----------|-----------------------------------------------------------------------------------------------|
| stable      | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git --tag v0.18.4` cargo-msrv |
| development | 💚        | `$ cargo install --git https://github.com/foresterre/cargo-msrv.git` cargo-msrv               |

#### [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

| cargo       | supported | command                                                     |
|-------------|-----------|-------------------------------------------------------------|
| stable      | 💚        | `$ cargo binstall --version 0.18.4 --no-confirm cargo-msrv` |
| development | ❌        |                                                             |

#### Arch Linux [extra repository](https://archlinux.org/packages/extra/x86_64/cargo-msrv/)

* `pacman -S cargo-msrv`

#### Docker

You can use the following commands for building and running via Docker:

* `$ docker build -t cargo-msrv .`
* `$ docker run -t -v "$(pwd)/Cargo.toml":/app/Cargo.toml cargo-msrv`

Docker images are also available on [Docker Hub](https://hub.docker.com/r/foresterre/cargo-msrv).

### Usage

<sup>⌘ [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

* [`cargo msrv find`](https://foresterre.github.io/cargo-msrv/commands/find.html)
  or [`cargo msrv find --linear`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a
  Cargo project in your current working directory.
* [`cargo msrv --path <dir> find`](https://foresterre.github.io/cargo-msrv/commands/find.html) to find the MSRV for a
  Cargo project in the `<dir>` directory.
* [`cargo msrv find -- <command>`](https://foresterre.github.io/cargo-msrv/commands/find.html) to use `<command>` as the
  compatibility check which decides whether a Rust version is
  compatible or not. This command should be runnable through rustup as `rustup run <toolchain> <command>`.
    * Example: `cargo msrv find -- cargo check --tests`.
    * Example: `cargo msrv find -- cargo check --ignore-rust-version`. Only available on Rust >= 1.56.
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

<sup>⌘ [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

JSON output may be enabled by providing the `--output-format json` flag: `cargo msrv find --output-format json`.
Events are printed as json lines. The event type is indicated by the `type` key.

Please see the [Output formats](https://foresterre.github.io/cargo-msrv/output-formats/index.html) and
[Output format: JSON](https://foresterre.github.io/cargo-msrv/output-formats/json.html) chapters of the
book for documentation of this output format.

### A short history

<sup>⌘ [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

_An [excerpt](https://foresterre.github.io/posts/puzzle-sharing-declarative-args-between-top-level-and-subcommand/)_

> `cargo-msrv` was originally born out of a desire to find the MSRV for a Rust project (more specifically package). MSRV
> stands for "minimal supported Rust version" and is the earliest or oldest version supported by a Rust project. For
> different projects this may mean different things, but for this post I will consider "support" as "does compile with a
> Rust toolchain of a certain version".
>
> Fast forward a few years, and the MSRV has become somewhat more ubiquitous which can also be seen by its inclusion
> into Cargo as the [rust-version](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field). Over
> time some additional tools were added to `cargo-msrv`. One of these was the `cargo msrv verify` subcommand.
>
> This subcommand can be used to check whether a Rust project supports its defined MSRV (e.g. via this `rust-version`
> field in the Cargo manifest). For example, in a CI pipeline you can use this to check whether your project works for
> the
> version you promised to your users.
>
> Originally, I kept the `cargo msrv` top level command aside from the subcommands for backwards compatibility reasons.
> In hindsight I probably shouldn't have done that, but as is, their coexistence at least provides me with the
> opportunity
> to write this blog post 😅.
>
> **How cargo msrv works**
>
>I described the "support" from "minimal supported Rust version" (MSRV) above as the somewhat simplified "does compile
> with a Rust toolchain of a certain version".
>
>You may write that as a function like so:  `fn is_compatible(version) -> bool`. If you run this test for some Rust
> version, when the function produces the value `true`, then we consider the Rust version to be supported. If instead
> the
> function produces the value `false`, then the Rust version is not supported.
>
>`cargo msrv` specifically searches for the _minimal_ Rust version which is supported by a given Rust project. While
> there are some caveats, we build upon
> Rust's [stability promise](https://blog.rust-lang.org/2014/10/30/Stability.html#committing-to-stability) . In our case
> that is the idea that Rust versions are backwards compatible.
>
>For a simple example to determine an MSRV, you can linearly walk backwards from the most recent Rust version to the
> earliest. When your project doesn't compile for a specific Rust version, then the last version that did compile can be
> considered your MSRV.
>
>Let's make it a bit more concrete with an example. For this example, we assume that Rust the following Rust versions
> exist: `1.0.0` up to and including `1.5.0`.
>
>Consider a project which uses the [Duration](https://doc.rust-lang.org/std/time/struct.Duration.html#) API which was
> stabilised by [Rust 1.3.0](https://github.com/rust-lang/rust/blob/master/RELEASES.md#version-130-2015-09-17) (and
> nothing more recent 😉).
>
>Then, if you would compile this project with Rust version from most recent to least recent, you would expect the
> following to happen:
>
>- `is_compatible(Rust 1.5.0)` returns `true` ✅
>- `is_compatible(Rust 1.4.0)` returns `true` ✅
>- `is_compatible(Rust 1.3.0)` returns `true` ✅
>- `is_compatible(Rust 1.2.0)` returns `false` ❌ ("Duration is not stable")
>- `is_compatible(Rust 1.1.0)` returns `false` ❌
>- `is_compatible(Rust 1.0.0)` returns `false` ❌
>
>Since we only care about the _minimal_ Rust version, you could have stopped searching after compiling Rust 1.2.0; Rust
> 1.3.0 was the earliest released Rust version which worked.
>
>In reality doing a linear search is quite slow (at the time of writing, there are 79 minor versions), so we primarily
> use a binary search instead to incrementally reduce the search space.
>
>`cargo msrv verify` works quite similar to "finding the MSRV", but instead of running a search which produces as
> primary output the MSRV, in this case the MSRV is already known in advance. So given a `MSRV` of `1.3.0` we just run
> the
`is_compatible(Rust 1.3.0)` function once. If it returns `true` we can say that the 1.3.0 is an acceptable MSRV (
> although not necessarily strictly so). More importantly, if it returns false, then the specified version is actually
> not
> supported, and thus can not be an MSRV).

### License

<sup>⌘ [preview](#preview) - [prerequisites](#prerequisites) - [install](#install) - [usage](#usage) - [JSON output format](#json-format) - [a short history](#a-short-history) - [license](#license)</sup>

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
