# Changelog

This is the changelog for [cargo-msrv](https://github.com/foresterre/cargo-msrv), a tool and cargo plugin, which can be
used to find the Minimum Supported Rust Version (MSRV) of your projects.
This changelog is aimed at users, and only contains user-relevant changes.

If you found an issue, have a suggestion or want to provide feedback or insights, please feel free to open an issue on
the [issue tracker](https://github.com/foresterre/cargo-msrv/issues), or open a topic in the [discussions section](https://github.com/foresterre/cargo-msrv/discussions).

## [Unreleased]

### Added

* Flag `--no-user-output` which disables user output.
* Subcommand `cargo msrv set`, which can be used to write a given MSRV to the Cargo manifest.

### Changed

* ⚠️ Breaking change: Changed default cargo-msrv (find) check command from `cargo check --all` to `cargo check`.
  * Revert to the old behaviour by running cargo-msrv with a custom check command: `cargo msrv -- cargo check --all`.

### Removed

* ⚠️ Breaking change: Value `void` was removed as a valid format for the `--output-format` option.

[Unreleased]: https://github.com/foresterre/cargo-msrv/compare/v0.14.2...HEAD


## [0.14.2] - 2022-02-09

* Fixed: Unable to set a custom check command when calling the cargo-msrv verify subcommand.

[0.14.2]: https://github.com/foresterre/cargo-msrv/compare/v0.14.1...v0.14.2

## [0.14.1] - 2022-02-02

* Fixed: Regression in the new bisection implementation introduced in v0.14.0, where the algorithm would stop one step too early.

[0.14.1]: https://github.com/foresterre/cargo-msrv/compare/v0.14.0...v0.14.1

## [0.14.0] - 2022-01-30

* Added: Verify as a subcommand
* Deprecated: Deprecated the `cargo msrv --verify` flag in favour of the `cargo msrv verify` subcommand.
* Changed: Changed terminology from 'Determine MSRV' to 'Find MSRV' for top level cargo-msrv command.
* Added: Flag `--linear` to the top cargo-msrv command ('find msrv') to choose explicitly for the `linear` search strategy.
* Changed: The default search method is now `bisect` instead of `linear`. ⚠
* Added: Feedback messages printed after each check, allowing users to see why certain Rust versions are not compatible.
* Added: Flag `--no-check-feedback` which disables the feedback messages printed after each check.


[0.14.0]: https://github.com/foresterre/cargo-msrv/compare/v0.13.0...v0.14.0

## [0.13.0] - 2021-12-25

* Fixed: Help text of the list subcommand will now be shown correctly .  
* Fixed: The json output of the list subcommand will now also report when it's done.
* Changed: Renamed list subcommand option "type" to "variant".
* Added: Subcommand "show" which shows the MSRV for the crate in your current working directory.
* Added: Logging to a file (default) or the stdout for developer information; can also be disabled entirely.
* Added: Output format option 'void', which disables user targeted output.
* Changed: `cargo-msrv` now exits with a non-zero exit code on failure.
* Changed: Clarify the reason when the program fails because of an IO error.
* Added: Book covering cargo-msrv topics, and its subcommands.
* Changed: Suggest `package.rust-version` is missing when MSRV can't be found in the Cargo manifest.
* Fixed: Return non-zero exit code when verify command fails.

[0.13.0]: https://github.com/foresterre/cargo-msrv/compare/v0.12.0...v0.13.0

## [0.12.0] - 2021-11-01

* Added subcommand `list` which lists the MSRV's of dependencies as specified by crate authors using the rust-version key.
* You can now also simply run cargo-msrv standalone, i.e. `cargo-msrv` instead of `cargo msrv`.

_Only list available as a subcommand. The determine-msrv and verify-msrv commands have not been ported to subcommands yet,
but are planned to._

[0.12.0]: https://github.com/foresterre/cargo-msrv/compare/v0.11.1...v0.12.0

## [0.11.1] - 2021-10-28

This release is equal to `v0.11.0`, except that the automated 'release build and packaging' task was fixed.  

[0.11.1]: https://github.com/foresterre/cargo-msrv/compare/v0.11.0...v0.11.1

## [0.11.0] - 2021-10-28

* Added `--release-source` which allows users to select a rust-releases source.
* A message is now shown to inform users when the index is being updated.
* Verify can now also read and use the `package.rust-version` MSRV key in the `Cargo.toml` manifest.
* When the `package.edition` is set, the first release supporting this edition will now be set as the minimum version.
This behaviour can be disabled by providing the `--no-read-min-edition` flag.

[0.11.0]: https://github.com/foresterre/cargo-msrv/compare/v0.10.0...v0.11.0

## [0.10.0] - 2021-10-01

* Add option to specify an edition alias instead of a minimum version.

[0.10.0]: https://github.com/foresterre/cargo-msrv/compare/v0.9.0...v0.10.0

# [0.9.0] - 2021-09-06

* cargo-msrv will no longer try to install upcoming, but unreleased, Rust releases.

[0.9.0]: https://github.com/foresterre/cargo-msrv/compare/v0.8.0...v0.9.0

## [0.8.0] - 2021-07-30

* Renamed `--minimum` to `--min` while keeping the former as an alias.
* Renamed `--maximum` to `--max` while keeping the former as an alias.

[0.8.0]: https://github.com/foresterre/cargo-msrv/compare/v0.7.0...v0.8.0

## [0.7.0] - 2021-06-15

* Added command (`--verify`) to verify the MSRV, if defined with the 'package.metadata.msrv' key in the `Cargo.toml`.

[0.7.0]: https://github.com/foresterre/cargo-msrv/compare/v0.6.0...v0.7.0

## [0.6.0] - 2021-05-27

* Added option,`--output-format`, to output the status of the program in a machine-readable format to stdout.

[0.6.0]: https://github.com/foresterre/cargo-msrv/compare/v0.5.0...v0.6.0

## [0.5.0] - 2021-05-13

* Added flag `--ignore-lockfile` which addresses an issue where newer lockfiles could not be parsed by older toolchains.

[0.5.0]: https://github.com/foresterre/cargo-msrv/compare/v0.4.0...v0.5.0

## [0.4.0] - 2021-04-09

* Added option to only take the latest patch Release version into acount, and make it the default.
* Added minimum and maximum release version options, which can be used to restrict the range of version to be checked.
* Change default check command to `cargo check -all` (was `cargo build --all`).
* Add option to output a `rust-toolchain` file.
* Add option to use a binary search instead of a linear search to find the MSRV.

[0.4.0]: https://github.com/foresterre/cargo-msrv/compare/v0.3.1...v0.4.0

## [0.3.1] - 2021-03-09

* Update deprecated dependency.
* Add repository to `Cargo.toml`.

[0.3.1]: https://github.com/foresterre/cargo-msrv/compare/v0.3.0...v0.3.1

## [0.3.0] - 2021-03-09

* Replace guessing Rust release version numbers with fetching an actual index.
* Make the terminal UI friendlier by replacing the log and wall of text with a progress bar and updating state.

[0.3.0]: https://github.com/foresterre/cargo-msrv/compare/v0.2.1...v0.3.0

## [0.2.1]

* Fix bug where no output was shown to the user by default.
* Increased own crate MSRV from `1.40.0` to `1.42.0`.

[0.2.1]: https://github.com/foresterre/cargo-msrv/compare/v0.2.0...v0.2.1

## [0.2.0]

* Replace `reqwest` http client with a smaller http client.
* Inform a user about sub-tasks such as installing a toolchain or running a check.
* Replace progress bar with logging based output.
* Increase own crate MSRV from `1.34.0` to `1.40.0`.
* Install rust targets with the `minimal` rustup profile.

[0.2.0]: https://github.com/foresterre/cargo-msrv/compare/v0.1.0...v0.2.0

## [0.1.0]

* Rust release channel manifest will now be re-fetched (expiry date of 1 day)
* Added support for custom rustup run commands; defaults to `cargo build --all`.
* Added support for custom toolchain targets.
* Added a spinner to show progression.

[0.1.0]: https://github.com/foresterre/cargo-msrv/compare/v0.0.0...v0.1.0

## [0.0.0]

* This was the initial `cargo-msrv` release.

[0.0.0]: https://github.com/foresterre/cargo-msrv/releases/tag/v0.0.0
