# [0.7.0] - 2021-06-15

* Added command (`--verify`) to verify the MSRV, if defined with the 'package.metadata.msrv' key in the 'Cargo.toml'

[0.7.0]: https://github.com/foresterre/cargo-msrv/compare/v0.6.0...v0.7.0

# [0.6.0] - 2021-05-27

* Added option (`--output-format`) to output the status of the program in a machine-readable format to stdout

[0.6.0]: https://github.com/foresterre/cargo-msrv/compare/v0.5.0...v0.6.0

# [0.5.0] - 2021-05-13

* Added new flag `--ignore-lockfile` which addresses an issue where newer lockfiles could not be parsed by older toolchains

[0.5.0]: https://github.com/foresterre/cargo-msrv/compare/v0.4.0...v0.5.0

# [0.4.0] - 2021-04-09

* Added option to only take the latest patch Release version into acount, and make it the default
* Added minimum and maximum release version options, which can be used to restrict the range of version to be checked
* Change default check command to `cargo check -all` (was `cargo build --all`)
* Add option to output a `rust-toolchain` file
* Add option to use a binary search instead of a linear search to find the MSRV

[0.4.0]: https://github.com/foresterre/cargo-msrv/compare/v0.3.1...v0.4.0

# [0.3.1] - 2021-03-09

* Update deprecated dependency
* Add repository to Cargo.toml

[0.3.1]: https://github.com/foresterre/cargo-msrv/compare/v0.3.0...v0.3.1

# [0.3.0] - 2021-03-09

* Replace guessing Rust release version numbers with fetching an actual index
* Make the terminal UI friendlier by replacing the log and wall of text with a progress bar and updating state

[0.3.0]: https://github.com/foresterre/cargo-msrv/compare/v0.2.1...v0.3.0

# [0.2.1]

* Fix bug where no output was shown to the user by default
* Increase own crate MSRV from 1.40.0 -> 1.42.0

[0.2.1]: https://github.com/foresterre/cargo-msrv/compare/v0.2.0...v0.2.1

# [0.2.0]

* Replace reqwest http client with a smaller http client
* Inform a user about sub-tasks such as installing a toolchain or running a check
* Replace progress bar with logging based output
* Increase own crate MSRV from 1.34.0 -> 1.40.0
* Install rust targets with the `minimal` rustup profile

[0.2.0]: https://github.com/foresterre/cargo-msrv/compare/v0.1.0...v0.2.0

# [0.1.0]

* Rust release channel manifest will now be refetched (expiry date of 1 day)
* Added support for custom rustup run commands; defaults to cargo build --all
* Added support for custom toolchain targets
* Added a spinner to show the process is ongoing

[0.1.0]: https://github.com/foresterre/cargo-msrv/compare/v0.0.0...v0.1.0

# [0.0.0]

* initial release

[0.0.0]: https://github.com/foresterre/cargo-msrv/releases/tag/v0.0.0
