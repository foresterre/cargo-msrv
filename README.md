# cargo-msrv

This crate can assist you in finding the Minimum Supported Rust Version for a crate.

### Install

`cargo install cargo-msrv` to install or <br>
`cargo install cargo-msrv --force` to update

### Usage

* `cargo msrv` to find the MSRV for the current working directory cargo project. 
* `cargo msrv --path <dir>` to find the MSRV in the `<dir>` directory cargo project.
* `cargo msrv -- <command> ` to use `<command>` as the compatibility check which decides whether a Rust version is
compatible or not. This command should be runnable through `rustup run <toolchain> <command>`.

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