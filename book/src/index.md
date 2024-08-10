# ✨ Introduction

`cargo-msrv` is a program which can help you find, set, show or verify the MSRV for a Rust crate. You can also list the
MSRV's of dependencies.

MSRV stands for 'Minimum Supported Rust Version', which is exactly what it says on the tin: the earliest
Rust release which a given Rust crate promises to support. Most often support for earlier Rust versions is
limited by newly introduced Rust language features, library functions or Rust editions.

For example, if you want to use const generics and be generic over integers, bool's or char's, you must use a Rust
compiler which supports the const generics MVP. This feature was introduced
in [Rust 1.51](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html#const-generics-mvp).
If you do not have any other code, or configuration, which requires an even newer Rust release, your MSRV would
be '1.51'.

While the MSRV has been a well-known concept within the Rust community for a long time, it was also introduced to the
Cargo build tool and package manager, as the `rust-version`
in [Cargo 1.56](https://github.com/rust-lang/cargo/blob/master/CHANGELOG.md#cargo-156-2021-10-21),
which is part of the [Rust 1.56](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html#cargo-rust-version) release
distribution.

In the [commands](./commands/index.md) section for more.

# 🔬 How it works

Cargo-msrv will test your project by running various Rust toolchains against your project. The order in which the
toolchains will be tested, and the amount of tests ran, depends on the search strategy, the set of available toolchains
and of course the limiting factor of the project which will determine the MSRV. We usually call each test a
cargo-msrv _check_. By default, the check command, the command used to test whether toolchain passes or fails a check,
is `cargo check`.

There are currently two search strategies: _linear_ and _bisect_ (default). When using the linear strategy, your crate
will be checked against toolchains from most-recent to least-recent. When a check fails, the previous Rust (if any)
version is returned as the MSRV (i.e. the highest toolchain for which a check command passes). The bisect strategy uses
a binary search to find the MSRV. This can be significantly faster, so it's usually advisable to keep it enabled by
default.

In addition to these two strategies, you can inspect the MSRV's set by the crate authors on which your project depends.
This is achieved by resolving the dependency graph of your crate, and querying each crate for its author specified MSRV.
Resolving the dependency graph is usually much quicker than running a toolchain command against your project, and may
give
you an indication of what your MSRV will be like. You can supply the highest listed version
as the `--min <version>` option: `cargo msrv --min <version>`. This will reduce the possible search space, and speed
up the search for the MSRV of your crate.

See [cargo-msrv find](./commands/find.md) and [cargo-msrv list](./commands/list.md) for more.

# 🥰 Thanks

Thanks for using cargo-msrv! If you found an issue, or have an issue request, or any other question, feel free to open
an issue at our GitHub [repository](https://github.com/foresterre/cargo-msrv/issues).

A special thanks goes to everyone who took the time to report an issue, discuss new features and contributed to the
documentation or the code! Thank you!
