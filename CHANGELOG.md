# [unreleased]

* Added `--release-source` which allows users to select a rust-releases source.
* A message is now shown to inform users when the index is being updated.
* Verify can now also read and use the `package.rust-version` MSRV key in the `Cargo.toml` manifest.
* When the `package.edition` is set, the first release supporting this edition will now be set as the minimum version.
This behaviour can be disabled by providing the `--no-read-min-edition` flag.

[unreleased]: https://github.com/foresterre/cargo-msrv/compare/v0.10.0...HEAD

# [0.10.0] - 2021-10-01

* Add option to specify an edition alias instead of a minimum version.

[0.10.0]: https://github.com/foresterre/cargo-msrv/compare/v0.9.0...v0.10.0

# [0.9.0] - 2021-09-06

* cargo-msrv will no longer try to install upcoming, but unreleased, Rust releases.

[0.9.0]: https://github.com/foresterre/cargo-msrv/compare/v0.8.0...v0.9.0

# [0.8.0] - 2021-07-30

* Renamed `--minimum` to `--min` while keeping the former as an alias.
* Renamed `--maximum` to `--max` while keeping the former as an alias.

[0.8.0]: https://github.com/foresterre/cargo-msrv/compare/v0.7.0...v0.8.0

# [0.7.0] - 2021-06-15

* Added command (`--verify`) to verify the MSRV, if defined with the 'package.metadata.msrv' key in the `Cargo.toml`.

[0.7.0]: https://github.com/foresterre/cargo-msrv/compare/v0.6.0...v0.7.0

# [0.6.0] - 2021-05-27

* Added option,`--output-format`, to output the status of the program in a machine-readable format to stdout.

[0.6.0]: https://github.com/foresterre/cargo-msrv/compare/v0.5.0...v0.6.0

# [0.5.0] - 2021-05-13

* Added flag `--ignore-lockfile` which addresses an issue where newer lockfiles could not be parsed by older toolchains.

[0.5.0]: https://github.com/foresterre/cargo-msrv/compare/v0.4.0...v0.5.0

# [0.4.0] - 2021-04-09

* Added option to only take the latest patch Release version into acount, and make it the default.
* Added minimum and maximum release version options, which can be used to restrict the range of version to be checked.
* Change default check command to `cargo check -all` (was `cargo build --all`).
* Add option to output a `rust-toolchain` file.
* Add option to use a binary search instead of a linear search to find the MSRV.

[0.4.0]: https://github.com/foresterre/cargo-msrv/compare/v0.3.1...v0.4.0

# [0.3.1] - 2021-03-09

* Update deprecated dependency.
* Add repository to `Cargo.toml`.

[0.3.1]: https://github.com/foresterre/cargo-msrv/compare/v0.3.0...v0.3.1

# [0.3.0] - 2021-03-09

* Replace guessing Rust release version numbers with fetching an actual index.
* Make the terminal UI friendlier by replacing the log and wall of text with a progress bar and updating state.

[0.3.0]: https://github.com/foresterre/cargo-msrv/compare/v0.2.1...v0.3.0

# [0.2.1]

* Fix bug where no output was shown to the user by default.
* Increased own crate MSRV from `1.40.0` to `1.42.0`.

[0.2.1]: https://github.com/foresterre/cargo-msrv/compare/v0.2.0...v0.2.1

# [0.2.0]

* Replace `reqwest` http client with a smaller http client.
* Inform a user about sub-tasks such as installing a toolchain or running a check.
* Replace progress bar with logging based output.
* Increase own crate MSRV from `1.34.0` to `1.40.0`.
* Install rust targets with the `minimal` rustup profile.

[0.2.0]: https://github.com/foresterre/cargo-msrv/compare/v0.1.0...v0.2.0

# [0.1.0]

* Rust release channel manifest will now be re-fetched (expiry date of 1 day)
* Added support for custom rustup run commands; defaults to `cargo build --all`.
* Added support for custom toolchain targets.
* Added a spinner to show progression.

[0.1.0]: https://github.com/foresterre/cargo-msrv/compare/v0.0.0...v0.1.0

# [0.0.0]

* This was the initial `cargo-msrv` release.

[0.0.0]: https://github.com/foresterre/cargo-msrv/releases/tag/v0.0.0
