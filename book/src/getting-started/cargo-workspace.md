### Cargo Workspace

If you're in a `Cargo` [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) while using `cargo-msrv`, you
can use the same flags `Cargo` offers to select a set of packages.

`cargo-msrv` aims to select packages like `cargo`. If you don't provide workspace specific flags, it will:
- in a virtual workspace every (default) member is selected
- at the root of a workspace with a root package the root is selected
- in a member directory, the member is selected

#### Selecting packages

`cargo-msrv` mirrors `cargo's` flags:

```shell
# Find the MSRV of every workspace member
cargo msrv find --workspace

# Find the MSRV of a specific workspace member
cargo msrv find --package $crate_name

# Verify every member, excluding one
cargo msrv verify --workspace --exclude $crate_name
```

#### Find and Verify

`cargo msrv find` finds the MSRV for each of the selected workspace packages. `cargo msrv verify` checks each package
against its defined MSRV, unless you provide a `--rust-version` (then, that will be the MSRV checked for every
selected package). 

#### Set

`cargo msrv set <version>` and `cargo msrv find --write-msrv` write the MSRV of each selected package to their own
manifests. `cargo msrv find --write-toolchain-file` writes the highest MSRV it found to the toolchain file at the
workspace root, since the toolchain file applies to the whole workspace.

If a package inherits its MSRV from the workspace (`rust-version.workspace = true`), `cargo-msrv` won't replace it.
If you want, you can set the workspace MSRV (i.e. `rust-version` in the `[workspace.package]` table of the workspace
root manifest) with:

```shell
cargo msrv set --workspace-msrv $version
```

#### Custom check commands

`cargo-msrv` isn't workspace aware when custom check commands are used. Custom check command can be anything (that can
be run with `rustup run <target> <cmd>`, and this includes non-cargo commands. `cargo-msrv` currently doesn't try to
infer that you provided a `cargo` custom check command, if you did.
 
For workspace crates it is recommended that you use `cargo-msrv` without custom check commands. If that's not possible,
please open a [topic](https://github.com/foresterre/cargo-msrv/discussions/categories/feature-requests) in the discussions
forum, and describe your use case.

#### Workspace support in cargo-msrv

`cargo-msrv` supports the following for a cargo workspace:

- Run `cargo msrv find` on a workspace to find the MSRV of the default workspace members
- Run `cargo msrv find --write-msrv` to write the found MSRV's of the default workspace members
- Run `cargo msrv verify` on a workspace, and verify the MSRV of the default workspace members
- Run `cargo msrv set --package <x>` to set the MSRV of a specific package in the workspace
- Run `cargo msrv show` on a workspace, and present the MSRV of all, or the selected workspace packages, to the user
- Run with `cargo msrv --workspace`, `cargo msrv --package <x>`, `cargo msrv --exclude <x>` flags to select workspace packages
  - Follows `cargo` conventions
- `cargo msrv find`, `cargo msrv verify` read from `workspace.package` [inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table), for example:
  - the `rust-version` field, used by `cargo msrv verify` to detect the MSRV to verify
  - the `edition` field, used by `cargo msrv find` to restrict the search space
  - the `include` and `exclude` fields which impact the workspace members
- Run `cargo msrv set --workspace-msrv <value>` to set the MSRV in the `workspace.package` table

Please open a [topic](https://github.com/foresterre/cargo-msrv/discussions/categories/feature-requests) if your use
case is not described in the above list.

##### Not supported yet

`cargo msrv list` doesn't operate on workspace packages yet. 
