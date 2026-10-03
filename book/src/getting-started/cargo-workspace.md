### Cargo Workspace

When developing a Rust project with cargo, you may use a cargo [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)
to manage a set of related packages together.

`cargo-msrv` selects packages like cargo. Without (workspace) flags, it acts on the package(s) cargo would select:
- in a virtual workspace every (default) member is selected
- at the root of a workspace with a root package the root is selected
- in a member directory, that member is selected

#### Selecting packages

You can pick the packages yourself. `cargo-msrv` mirrors these flags from cargo:

```shell
# Find the MSRV of every workspace member
cargo msrv find --workspace

# Find the MSRV of a single member
cargo msrv find --package $crate_name

# Verify every member, except one
cargo msrv verify --workspace --exclude $crate_name
```

`cargo msrv find` and `cargo msrv verify` check each selected package on its own, by running
`cargo check --package <name>`. `cargo msrv verify` checks each package against its own MSRV, unless you give a
`--rust-version`, which then applies to every selected package.

#### Writing the MSRV

`cargo msrv set <version>` and `cargo msrv find --write-msrv` write the MSRV of each selected package to its own
manifest. `cargo msrv find --write-toolchain-file` writes the highest MSRV it found to the toolchain file at the
workspace root, since the toolchain file applies to the whole workspace.

If a package inherits its MSRV from the workspace (`rust-version.workspace = true`), `cargo-msrv` won't replace it.
Instead, set the MSRV for every member at once:

```shell
cargo msrv set --workspace-root $version
```

This writes `rust-version` to the `[workspace.package]` table of the workspace root manifest.

#### Custom check commands

When you use a custom check command, for example `cargo msrv find -- cargo check --all-targets`, `cargo-msrv` can't
add `--package` to it. The command then runs once for the whole selection, and the result is reported for the
workspace as a whole.

#### Workspace support in cargo-msrv

`cargo-msrv` supports the following for a cargo workspace:

- Run `cargo msrv find` on a workspace, and find the MSRV of all, or the selected workspace packages
- Run `cargo msrv find --write-msrv` to write the found MSRV's of the selected workspace packages
- Run `cargo msrv verify` on a workspace, and verify the MSRV of all, or the selected workspace packages
- Run `cargo msrv set --package <x>` to set the MSRV of a specific package in the workspace
- Run `cargo msrv show` on a workspace, and present the MSRV of all, or the selected workspace packages, to the user
- Add `cargo msrv --workspace`, `cargo msrv --package <x>`, `cargo msrv --exclude <x>` flags to select workspace packages
  - User selection of workspace packages was added in [#1025](https://github.com/foresterre/cargo-msrv/pull/1025/files)
  - JSON reporting of the selected workspace was added in [#1030](https://github.com/foresterre/cargo-msrv/pull/1030/files) 
- `cargo msrv find`, `cargo msrv verify` and others should support `workspace.package` [inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table), for example for:
  - the `rust-version` field, used by `cargo msrv verify` to detect the MSRV to verify
  - the `edition` field, used by `cargo msrv find` to restrict the search space
  - the `include` and `exclude` fields to define the workspace members
- Run `cargo msrv set --workspace-root <value>` to set the MSRV in the `workspace.package` table

The following features are under consideration:
- Run `cargo msrv list` on a workspace to list the MSRV of dependencies of each of the workspace crates.  

Please open an [issue](https://github.com/foresterre/cargo-msrv/issues) if your use case is not described in the above list.

#### Follow progress on GitHub

Tracking issue: [#1026](https://github.com/foresterre/cargo-msrv/issues/1026)

**cargo msrv find &amp; cargo msrv verify**

- [Add --workspace flag to subcommand find #873](https://github.com/foresterre/cargo-msrv/issues/873)

**cargo msrv list**

- No dedicated issue yet

**cargo msrv set**

- No dedicated issue yet

**cargo msrv show**

- [cargo msrv show should show all workspace crate MSRV's #1024](https://github.com/foresterre/cargo-msrv/issues/1024)
