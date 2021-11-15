🚧 Section is work-in-progress.

# ✨ Introduction

`cargo-msrv` is a program which can help you find the MSRV for a Rust crate.

MSRV stands for 'Minimum Supported Rust Version', which is exactly what it says on the tin: the earliest
Rust release which a given Rust crate promises to support. Most often support for earlier Rust versions is
limited by newly introduced Rust language features, library functions or Rust editions.

For example, if you want to use const generics and be generic over integers, bool's or char's, you must use a Rust
compiler which supports the const generics MVP. This feature was introduced in [Rust 1.51](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html#const-generics-mvp),
which. If you then do not have any code, or configuration which requires an even newer Rust release, your MSRV would
be '1.51'.

While the MSRV has been a well-known concept within the Rust community for a long time, it was also introduced to the
Cargo build tool and package manager, as the `rust-version` in [Cargo 1.56](https://github.com/rust-lang/cargo/blob/master/CHANGELOG.md#cargo-156-2021-10-21),
which is part of the [Rust 1.56](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html#cargo-rust-version) release
distribution.

# 🔬 How it works

⚠️work-in-progress

# 🥰 Thanks

Thanks for using cargo-msrv! If you found an issue, or have an issue request, or any other question, feel free to open
an issue at: report the issue at our Github [repository](https://github.com/foresterre/cargo-msrv/issues).

A special thanks goes to everyone who took the time to report an issue, discuss new features and contributed to the
documentation or the code! Thank you!