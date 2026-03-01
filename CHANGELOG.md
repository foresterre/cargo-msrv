# Changelog

This is the changelog for [cargo-msrv](https://github.com/foresterre/cargo-msrv), a tool and cargo plugin, which can be
used to find the Minimum Supported Rust Version (MSRV) of your projects.
This changelog is aimed at users, and only contains user-relevant changes.

If you found an issue, have a suggestion or want to provide feedback or insights, please feel free to open an issue on
the [issue tracker](https://github.com/foresterre/cargo-msrv/issues), or open a topic in
the [discussions section](https://github.com/foresterre/cargo-msrv/discussions).

## Unreleased

### Added

* Added `--skip-unavailable-toolchains` which treats a Rust version as incompatible when a toolchain failed to install or was otherwise unavailable

### Fixed

#### Infra

* Fix missing rustup dependency in docker image

## 0.18.4 - 2025-03-28

### Changed

* Improve compatibility of path canonicalization on Windows
* Restore lockfile moved by `--ignore-lockfile`, after a user interrupts the process

## 0.18.3 - 2025-03-12

#### Infra

* Attempt to fix release pipeline for all binaries (license generation step failed)

## 0.18.2 - 2025-03-12

### Security

* Update transitive dependency `ring` to `>= 0.17.12` (Resolves RUSTSEC-2025-0009, `cargo-msrv` is unlikely to be
  affected)

## 0.18.1 - 2025-02-18

### Fixed

#### Infra

* Fix release pipeline for `x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl` binaries

## 0.18.0 - 2025-02-17

### Added

* Fetching the rust-releases index now attempts to set proxy settings detected from the environment (HTTP_PROXY)

## 0.17.1 - 2024-11-24

### Fixed

#### Infra

* Fix release pipeline for binaries

## 0.17.0 - 2024-11-21

### Added

* Rust edition 2024 can now be detected

## 0.16.3 - 2024-11-11

### Added

* Added `--workspace`, `--all`, `--package` and `--exclude` CLI options for package selection when in a Cargo project
  (currently, limitations apply to support for Cargo workspaces)
* `cargo msrv find --write-msrv` is now aliased by `cargo msrv find --set`, which is consistent in terminology with
  `cargo msrv set`

### Changed

* Toolchains installed with `rustup install` now use the `--no-self-update` flag

## 0.16.2 - 2024-10-10

### Fixed

#### Infra

* Fix release pipeline for `x86_64-unknown-linux-musl` binaries

## 0.16.1 - 2024-10-09

### Added

#### Infra

* Build and release `x86_64-unknown-linux-musl` binaries

## 0.16.0 - 2024-10-09

### Added

* **Added `cargo msrv find` subcommand to determine the MSRV (this subcommand was moved from the top level `cargo msrv`
  command to its own subcommand)**
* Added options `--ignore-lockfile`,  `--no-check-feedback`, `--target`, `--component`, `--features`, `--all-features`,
  `--no-default-features` and the last argument "custom compatibility check command", which were previously available
  from the top level `cargo msrv` command to the `cargo msrv verify` subcommand.
* Subcommand `cargo msrv verify` now supports setting a custom Rust version via the `--rust-version <VERSION>` argument,
  which can be used to check for a crate's compatibility against a specific Rust version.
* Added flag `--write-msrv` to cargo msrv (find), which upon finding the MSRV writes its value to the Cargo manifest.
* Added option to refer to a specific crate using its Cargo manifest (with `--manifest-path`) instead of its path (
  with `--path`)
* Added a 'minimal' output option intended for machine-readable use when full json output is undesirable.
* Added `--features` option, `--all-features` flag and `--no-default-features` flag, which are forwarded to the default
  compatibility check command
* Added `--component` option, which can be used to run cargo msrv find or verify with one or more Rust toolchain
  components.
* `cargo msrv verify` now supports
  Cargo [workspace inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table), and will
  now correctly inherit the MSRV (i.e. `package.rust-version`) defined by a workspace

### Changed

* CLI options are now grouped.
* Option `--min <version>` now also accepts two component semver `major.minor` versions, in addition to full three
  component (strict) SemVer versions, and edition specifiers like "2015", "2018" and "2021".
* Option `--max <version>` now also accepts two component semver `major.minor` versions, in addition to full three
  component (strict) SemVer versions.
* The rust-releases index is now only fetched for subcommands which depend on it.
* Renamed `--toolchain-file` to `--write-toolchain-file` to emphasise that the toolchain-file is an output.
* Subcommand `cargo msrv set` will now default to writing a regular TOML table for the metadata MSRV fallback value,
  instead of an inline table.
* The rust-toolchain file will now be overwritten if a rust-toolchain file was already present.
* Updated user output formatting to be more consistent between output formats.
* `cargo-msrv` now requires paths to be UTF-8.
* `--write-msrv` now writes two, instead of three component version numbers.

#### Infra

* Changed release artifact name of `cargo-msrv` packages on Github, such that they can be installed
  with `cargo-binstall` out of the box.

### Fixed

* Subcommand `cargo msrv set` will now return an error when the Cargo manifest solely consists of a virtual workspace.
* The program will no longer return an unformatted message when a command failed and the output format was set to json.
* Fix issue where reading the fallback MSRV from a TOML inline table was not possible.
* Fix an index out-of-bounds panic which occurred if the filtered Rust releases search space was empty.
* Use compilation target instead of build machine target for MSRV checks.
* Fix issue where `--manifest-path Cargo.toml` would yield an empty manifest path.
* Supply provided components to `verify` subcommand.
* The CLI arguments `--target` and `--component` were previously inadvertently ignored when provided
  to `cargo msrv verify`.
* Fixed issue where some errors were not being reported (e.g. `cargo msrv verify` did not print an error if it wasn't
  possible to resolve the MSRV to check against).

### Removed

* Removed deprecated option `cargo msrv --verify`. Use `cargo msrv verify` instead.
* Removed option to disable filtering the Rust releases search space by the Rust edition in from the Cargo
  manifest, `--no-read-min-edition`.
* Moved the top level `cargo msrv` "find the MSRV" action to the `cargo msrv find` subcommand, which removed several
  options and flags from the top level command which had previously no effect on other subcommands.

## 0.15.1 - 2022-02-24

In this release a license generation step was fixed, which caused the automated binary builds to fail.
In addition, a Cargo feature toggle was added which adds a feature gate for the rust-dist release source of
the rust-releases crate. This allows people who build from source to build cargo-msrv without the default
features (i.e. without rust-dist), which reduces build time and binary size.

This release does not contain user-facing changes, hence the lack of changelog entries for this version.

## 0.15.0 - 2022-02-23

### Added

* Flag `--no-user-output` which disables user output.
* Subcommand `cargo msrv set`, which can be used to write a given MSRV to the Cargo manifest.

### Changed

* ⚠️ Breaking change: Changed default cargo-msrv (find) check command from `cargo check --all` to `cargo check`.
    * To revert to the old behaviour, run cargo-msrv with the following custom check
      command: `cargo msrv -- cargo check --all`.

### Removed

* ⚠️ Breaking change: Value `void` was removed as a valid format for the `--output-format` option.

## 0.14.2 - 2022-02-09

* Fixed: Unable to set a custom check command when calling the cargo-msrv verify subcommand.

## 0.14.1 - 2022-02-02

* Fixed: Regression in the new bisection implementation introduced in v0.14.0, where the algorithm would stop one step
  too early.

## 0.14.0 - 2022-01-30

* Added: Verify as a subcommand
* Deprecated: Deprecated the `cargo msrv --verify` flag in favour of the `cargo msrv verify` subcommand.
* Changed: Changed terminology from 'Determine MSRV' to 'Find MSRV' for top level cargo-msrv command.
* Added: Flag `--linear` to the top cargo-msrv command ('find msrv') to choose explicitly for the `linear` search
  strategy.
* Changed: The default search method is now `bisect` instead of `linear`. ⚠
* Added: Feedback messages printed after each check, allowing users to see why certain Rust versions are not compatible.
* Added: Flag `--no-check-feedback` which disables the feedback messages printed after each check.

## 0.13.0 - 2021-12-25

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

## 0.12.0 - 2021-11-01

* Added subcommand `list` which lists the MSRV's of dependencies as specified by crate authors using the rust-version
  key.
* You can now also simply run cargo-msrv standalone, i.e. `cargo-msrv` instead of `cargo msrv`.

_Only list available as a subcommand. The determine-msrv and verify-msrv commands have not been ported to subcommands
yet,
but are planned to._

## 0.11.1 - 2021-10-28

This release is equal to `v0.11.0`, except that the automated 'release build and packaging' task was fixed.

## 0.11.0 - 2021-10-28

* Added `--release-source` which allows users to select a rust-releases source.
* A message is now shown to inform users when the index is being updated.
* Verify can now also read and use the `package.rust-version` MSRV key in the `Cargo.toml` manifest.
* When the `package.edition` is set, the first release supporting this edition will now be set as the minimum version.
  This behaviour can be disabled by providing the `--no-read-min-edition` flag.

## 0.10.0 - 2021-10-01

* Add option to specify an edition alias instead of a minimum version.

## 0.9.0 - 2021-09-06

* cargo-msrv will no longer try to install upcoming, but unreleased, Rust releases.

## 0.8.0 - 2021-07-30

* Renamed `--minimum` to `--min` while keeping the former as an alias.
* Renamed `--maximum` to `--max` while keeping the former as an alias.

## 0.7.0 - 2021-06-15

* Added command (`--verify`) to verify the MSRV, if defined with the 'package.metadata.msrv' key in the `Cargo.toml`.

## 0.6.0 - 2021-05-27

* Added option,`--output-format`, to output the status of the program in a machine-readable format to stdout.

## 0.5.0 - 2021-05-13

* Added flag `--ignore-lockfile` which addresses an issue where newer lockfiles could not be parsed by older toolchains.

## 0.4.0 - 2021-04-09

* Added option to only take the latest patch Release version into account, and make it the default.
* Added minimum and maximum release version options, which can be used to restrict the range of version to be checked.
* Change default check command to `cargo check -all` (was `cargo build --all`).
* Add option to output a `rust-toolchain` file.
* Add option to use a binary search instead of a linear search to find the MSRV.

## 0.3.1 - 2021-03-09

* Update deprecated dependency.
* Add repository to `Cargo.toml`.

## 0.3.0 - 2021-03-09

* Replace guessing Rust release version numbers with fetching an actual index.
* Make the terminal UI friendlier by replacing the log and wall of text with a progress bar and updating state.

## 0.2.1

* Fix bug where no output was shown to the user by default.
* Increased own crate MSRV from `1.40.0` to `1.42.0`.

## 0.2.0

* Replace `reqwest` http client with a smaller http client.
* Inform a user about sub-tasks such as installing a toolchain or running a check.
* Replace progress bar with logging based output.
* Increase own crate MSRV from `1.34.0` to `1.40.0`.
* Install rust targets with the `minimal` rustup profile.

## 0.1.0

* Rust release channel manifest will now be re-fetched (expiry date of 1 day)
* Added support for custom rustup run commands; defaults to `cargo build --all`.
* Added support for custom toolchain targets.
* Added a spinner to show progression.

## 0.0.0

* This was the initial `cargo-msrv` release.
