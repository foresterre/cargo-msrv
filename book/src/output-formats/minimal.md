# Output format: minimal

The purpose of the `minimal` output format option is to provide just enough output for a machine (or shell script!)
to be understandable, while not requiring elaborate parsers like the `json` output format. It may also be used as
a minimal human-readable format.

This output format can be summarized by the following two statements:

* If the command was successful, it prints the commands final result and exits with a zero exit code. Output is printed
  to `stdout`.
* If the command was unsuccessful, it prints an error message, and exits with a non-zero exit code. Output is printed  
  to `stderr`.

You may also refer to the 🚧 TODO 🚧 section to determine which kind of errors result in a non-zero
exit code, and how different errors are categorised.

# Output by subcommand

## \# cargo msrv (find)

If the MSRV was found, we report this minimal supported Rust version by writing it to `stdout`.
If it could not be found, we report `none` instead, and write this value to `stderr`.

### Example 1

If the MSRV is `1.60.0`, the output will be just `1.60.0`.

```shell
$ cargo msrv find --output-format minimal
# stdout
1.60.0
```

### Example 2

If the MSRV can't be found, for example if your project requires a nightly compiler feature
or has incorrect syntax, the output will be `none`.

```shell
$ cargo msrv find --output-format minimal
# stderr
none
```

## \# cargo msrv list

The `list` subcommand is not supported by the `minimal` output format, and "unsupported" will be printed.
Support may be added in the future.

## \# cargo msrv set

The `set` subcommand prints the version set as MSRV.

### Example 1

If we set our MSRV to be `1.31`, the output will be `1.31`.

```shell
cargo msrv find --output-format minimal set 1.31
# stdout
1.31
```

## \# cargo msrv show

The `show` subcommand prints the detected MSRV, if specified in the Cargo Manifest.

### Example 1

Assuming our Cargo manifest contains a `1.60` MSRV, `cargo-msrv` will print `1.60`.

**Cargo.toml**

```toml
[package]
rust-version = "1.60"
```

**Shell**

```shell
$ cargo msrv find --output-format minimal show
# stdout
1.60
```

### Example 2

Assuming our Cargo manifest lists the MSRV in the `package.metadata.msrv` field, `cargo-msrv` will print `1.21.0`.
The `package.rust-version` field has precedence over the `package.metadata.msrv`. You may see the
`package.metadata.msrv`
key for crates which use a Cargo version which does not yet support the `package.rust-version` field. `cargo-msrv`
supports both fields.

**Cargo.toml**

```toml
[package.metadata]
msrv = "1.21.0"
```

**Shell**

```shell
$ cargo msrv find --output-format minimal show
# stdout
1.21.0
```

## \# cargo msrv verify

The `verify` subcommand prints `true` (to stdout) with exit code zero if checking the toolchain for this platform which
matches the MSRV succeeds. Else, it prints `false` (to stderr) with an exit code which is non-zero.

### Example 1

Assuming our Cargo manifest contains an MSRV definition in the `package.rust-version` or `package.metadata.msrv` field,
and the compatibility check succeeds:

```toml
[package.metadata]
msrv = "1.31"
```

**Shell**

```shell
$ cargo msrv find --output-format minimal verify
# stdout
true
```

### Example 2

Assuming the given crate is incompatibility with the given MSRV:

**Shell**

```shell
$ cargo msrv find --output-format minimal verify --rust-version 1.31
# stderr
false
```
