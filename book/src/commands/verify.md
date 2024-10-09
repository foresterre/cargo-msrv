# cargo-msrv verify

# COMMAND

* Standalone: `cargo-msrv verify`
* Through Cargo: `cargo msrv verify`

# PREVIEW

[![asciicast](https://asciinema.org/a/679863.svg)](https://asciinema.org/a/679863)

# DESCRIPTION

Verify whether the MSRV can be satisfied.

The MSRV can be specified in the Cargo manifest (`Cargo.toml`) using either the `package.rust-version` (Rust >=1.56,
recommended),
or the `package.metadata.msrv` field.

If the check fails, the program returns with a non-zero exit code.

# OPTIONS

**`--rust-version` version**

Specify the Rust version of a Rust toolchain, against which the crate will be checked for compatibility.

# EXAMPLES

1. Verify whether the MSRV specified in the Cargo manifest is satisfiable (Good case).

Given a minimal rust crate with the following Cargo.toml manifest:

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2021"
rust-version = "1.56.0"
```

and this minimal lib.rs file:

```rust
fn main() {
    println!("Hello world");
}
```

We check whether the MSRV's check command, in this case the default `cargo check`, can be satisfied.
The crate author specified the MSRV in the Cargo.toml, using the `package.rust-version` key.
Since the example crate used no features requiring a more recent version than Rust 1.56, the check will be satisfied,
and the program returns a with exit code 0 (success).

```shell
cargo msrv verify # Will succeed, and return with exit code 0
```

2. Verify whether the MSRV specified in the Cargo manifest is satisfiable (Bad case).

Given a minimal rust crate with the following Cargo.toml manifest:

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2021"
rust-version = "1.56.0"
```

and this minimal lib.rs file:

```rust
fn main() {
    let cmd = Command::new("ls");
    assert_eq!(cmd.get_program(), "ls"); // will fail because Command::get_program was introduced in 1.57, which is greater than 1.56 (the MSRV)
}
```

We check whether the MSRV's check command, in this case the default `cargo check`, can be satisfied.
The crate author specified the MSRV in the Cargo.toml, using the `package.rust-version` key.
Since the example crate used a feature requiring a more recent version than Rust 1.56, the check cannot be satisfied,
and the program returns a with a non-zero exit code (failure).

```shell
cargo msrv verify # Will fail, and return a non-zero exit code
```

3. Run the 'verify' subcommand on a crate not in our current working directory.

```shell
cargo msrv --path path/to/my/crate verify
```

This example shows how to use arguments (in this case `--path`) shared between the default cargo-msrv command and
verify.
Note that shared arguments must be specified before the subcommand (here `verify`).

4. Run the 'verify' subcommand using a self-determined Rust version.

```shell
cargo msrv verify --rust-version 1.56
```

