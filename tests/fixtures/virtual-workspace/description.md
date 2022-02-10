The `virtual-workspace` fixture consists of a virtual workspace, and two packages within the virtual workspace named 'a'
and `b`. The package `a` has a defined `rust-version` of `1.56` and package `b` has a defined `rust-version` of `1.58`.

When determining the MSRV for package `a` in the workspace, e.g. by running `cargo msrv --path a` (with default check
command `cargo check`), it should find an MSRV of `1.56`, while running `cargo msrv --path b` should return an MSRV of
`1.58`.

Prior to cargo-msrv 0.15.0, the default check command was `cargo check --all`. This would instead always find the
greatest common MSRV of the workspace, regardless of which package we were in. We can simulate this behaviour by
running `cargo msrv -- cargo check --all`. The result of this command instead should be an MSRV of `1.58`, as package
`b` is the greatest common MSRV in the workspace.