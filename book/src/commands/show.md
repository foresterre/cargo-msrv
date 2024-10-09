# cargo-msrv show

# COMMAND

* Standalone: `cargo-msrv show`
* Through Cargo: `cargo msrv show`

# PREVIEW

[![asciicast](https://asciinema.org/a/679864.svg)](https://asciinema.org/a/679864)

# DESCRIPTION

Print the crate author specified MSRV.

This is either the `package.rust-version` field or the `package.metadata.msrv` field in the Cargo manifest (
`Cargo.toml`).

<!-- # OPTIONS -->

# EXAMPLES

1. Show the MSRV specified by a crate author

```shell
cargo msrv show
```
