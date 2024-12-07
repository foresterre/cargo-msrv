### Cargo Workspace

When developing a Rust project with cargo, you may use a cargo [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)
to manage a set of related packages together.

`cargo-msrv` currently partially supports cargo workspaces although full support is on the way.

#### Finding the MSRV of a workspace member

To find the MSRV of a workspace crate, you can run:

```shell
cargo msrv find -- cargo check -p $crate_name
```

To verify the MSRV of a workspace, you can run:

```shell
cargo msrv verify -- cargo check -p $crate_name
```

#### Workspace support in cargo-msrv

`cargo-msrv` should support the follow for a cargo workspace:

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

The following features are under consideration:
- Run `cargo msrv set --workspace <value>` on a workspace to set a common MSRV
- Run `cargo msrv set --workspace-package <x>` to set the MSRV to the workspace.package table, if in a workspace
    - TODO: determine the name of the flag
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
