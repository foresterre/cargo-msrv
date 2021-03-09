# Unreleased

* Replace guessing Rust release version numbers with fetching an actual index
* Improve terminal UI: replace logger ui with a progress bar console ui

# v0.2.1

* Fix bug where no output was shown to the user by default
* Increase own crate MSRV from 1.40.0 -> 1.42.0

# v0.2.0

* Replace reqwest http client with a smaller http client
* Inform a user about sub-tasks such as installing a toolchain or running a check
* Replace progress bar with logging based output
* Increase own crate MSRV from 1.34.0 -> 1.40.0
* Install rust targets with the `minimal` rustup profile

# v0.1.0

* Rust release channel manifest will now be refetched (expiry date of 1 day)
* Added support for custom rustup run commands; defaults to cargo build --all
* Added support for custom toolchain targets
* Added a spinner to show the process is ongoing

# v0.0.0

* initial release
