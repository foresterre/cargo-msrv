# cargo-msrv set

# COMMAND

* Standalone: `cargo-msrv set`
* Through Cargo: `cargo msrv set`

# DESCRIPTION

Set the MSRV in the Cargo manifest.

This is either the `package.rust-version` field or the `package.metadata.msrv` field in the Cargo manifest (`Cargo.toml`).

<!-- # OPTIONS -->

# EXAMPLES

1. Set an MSRV by providing a two component Rust version

```shell
cargo msrv set 1.56
```

2. Set an MSRV by providing a three component Rust version

```shell
cargo msrv set 1.58.1
```
