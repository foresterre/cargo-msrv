# cargo-msrv

This crate can assist you in finding the Minimum Supported Rust Version for a crate.

### Install

With Cargo from crates.io [latest release]:

`cargo install cargo-msrv` to install or <br>
`cargo install cargo-msrv --force` to update

With Cargo from Github [latest development version]:

`cargo install cargo-msrv --git https://github.com/foresterre/cargo-msrv.git --branch main`


### Preview

<!-- old preview: ![preview](https://i.imgur.com/5hAzJQu.gif) -->
![preview](https://user-images.githubusercontent.com/5955761/113495231-aa41f800-94df-11eb-87ba-9eb852dee8b7.gif)



### Usage

* `cargo msrv` to find the MSRV for the current working directory cargo project. 
* `cargo msrv --path <dir>` to find the MSRV in the `<dir>` directory cargo project.
* `cargo msrv -- <command> ` to use `<command>` as the compatibility check which decides whether a Rust version is
compatible or not. This command should be runnable through `rustup run <toolchain> <command>`.

**Options:**
```
cargo-msrv
Helps with finding the Minimal Supported Rust Version (MSRV)

USAGE:
    cargo msrv [OPTIONS]

OPTIONS:
        --bisect
            Use a binary search to find the MSRV instead of a linear search

    -h, --help
            Prints help information

        --include-all-patch-releases
            Include all patch releases, instead of only the last

        --maximum <max>
            Latest version to take into account

        --minimum <min>
            Defaults to 1.31.0 which is the first Rust 2018 compatible version [default: 1.31.0]

        --path <DIR>
            Path to the cargo project directory.

        --target <TARGET>
            Check against a custom target (instead of the rustup default)

        --toolchain-file
            The toolchain file will pin the Rust version for this crate. See https://rust-
            lang.github.io/rustup/overrides.html#the-toolchain-file for more
    -V, --version
            Prints version information


ARGS:
    <COMMAND>...
            If given, this command is used to validate if a Rust version is compatible. Should be available to rustup,
            i.e. the command should work like so: `rustup run <toolchain> <COMMAND> The default check action is `cargo
            build --all`

If arguments are provided after two dashes (`--`), they will be used as a custom command to validate whether a Rust
version is compatible. By default for this validation the command `cargo build` is used. Commands should be runnable by
rustup, i.e. validation commands will be passed to rustup like so: `rustup run <toolchain> <COMMAND...>`. You'll only
need to provide the <COMMAND...> part.
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
