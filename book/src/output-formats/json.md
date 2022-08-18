# Output format: json

The `json` output format is intended to be used as a machine-readable output, to be interpreted by tooling, altough
humans may also use it, as it provides the most detailed output of all supported output formats.

As described on the [output-formats](index.md) page, `cargo-msrv` reports the status of the program via
events. A processor transforms these events into their output-format. In case of the `json` output format, 
events are almost 1-on-1 serialized to json (there are a few exceptions), and then printed to `stderr`.
Each json serialized event ends with a newline. Each line thus represents a single serialized event. 

To use the `json` output format, run `cargo-msrv` with the `--output-format json` option. 
For example, if you want to find the MSRV, you could run `cargo msrv --output-format json`.


In the next section, you can find a description of the common fields of events.
The section thereafter gives an overview of each of the supported events, with for each event its event specific fields.

# Common fields on events

| name  | optional | values           | description                              |
|-------|----------|------------------|------------------------------------------|
| type  | no       |                  | Identifies a specific event              |
| scope | yes      | "start" or "end" | Marks the begin or end of a scoped event |

The **type** field can be used to identify a specific event.

The **scope** field is only present for scoped events. The `start` value marks the start of a scoped event, while `end`
marks the end of a scoped event. The scope is not an inherent property of the event itself. A scope adds a span during
which an event took place.


# Events

## Event: Meta

**type:** meta

**description:** Reports metadata about the currently running `cargo-msrv` program instance.

**fields:**

| name           | description                                                                                             |
|----------------|---------------------------------------------------------------------------------------------------------|
| instance       | Name of the running `cargo-msrv` program as defined on compile time. This will usually be `cargo-msrv`. |
| version        | Version of the running `cargo-msrv` program as defined on compile time.                                 |
| sha_short      | Short SHA hash of the git commit used to compile and build `cargo-msrv`.                                |
| target_triple  | Target triple of toolchain used to compile and build `cargo-msrv`.                                      |
| cargo_features | Features which were enabled during the compilation of `cargo-msrv`.                                     |
| rustc          | Version of `rustc` used to compile `cargo-msrv`.                                                        |

**example:**

```json lines
{"type":"meta","instance":"cargo-msrv","version":"0.15.1","sha_short":"79582b6","target_triple":"x86_64-pc-windows-msvc","cargo_features":"default,rust_releases_dist_source","rustc":"1.62.0"}
```

## Event: FetchIndex

**type:** fetch_index

**description:** Prior to determining the MSRV of a crate, we have to figure out which Rust versions are available. 
We obtain those using the [rust-releases](https://crates.io/crates/rust-releases) library. The `FetchIndex` event
reports that the index is being fetched, and details which source is used. 

**fields:**

| name           | description                                               |
|----------------|-----------------------------------------------------------|
| source         | Place from where the available Rust releases are obtained |

**example:**

```json lines
{"type":"fetch_index","source":"rust_changelog","scope":"start"}
{"type":"fetch_index","source":"rust_changelog","scope":"end"}
```

## Event: CheckToolchain

**type:** check_toolchain

**description:** The primary way for `cargo-msrv` to determine whether a given Rust toolchain is compatible with your
crate, is by installing a toolchain and using it to check a crate for compatibility with this toolchain. The
`CheckToolchain` event is wrapped around this process and notifies you about the start and end of this process.
This event is called as a scoped event, and within it's scope, you'll find the following events: `setup_toolchain`,
`check_method` and `check_result`, which are described in more detail below. 

**fields:**

| name              | description                              |
|-------------------|------------------------------------------|
| toolchain         | The toolchain to be located or installed |
| toolchain.version | The Rust version of the toolchain        |
| toolchain.target  | The target-triple of the toolchain       |

**example:**

```json lines
{"type":"check_toolchain","toolchain":{"version":"1.35.0","target":"x86_64-pc-windows-msvc"},"scope":"start"}
{"type":"check_toolchain","toolchain":{"version":"1.35.0","target":"x86_64-pc-windows-msvc"},"scope":"end"}
```

## Event: SetupToolchain

**type:** setup_toolchain

**description:** The primary way for `cargo-msrv` to determine whether a given Rust toolchain is compatible with your
crate, is by installing a toolchain and using it to check a crate for compatibility with this toolchain. The
`SetupToolchain` event reports about the process of locating or installing a given toolchain.

**fields:**

| name              | description                              |
|-------------------|------------------------------------------|
| toolchain         | The toolchain to be located or installed |
| toolchain.version | The Rust version of the toolchain        |
| toolchain.target  | The target-triple of the toolchain       |

**example:**

```json lines
{"type":"setup_toolchain","toolchain":{"version":"1.47.0","target":"x86_64-pc-windows-msvc"},"scope":"start"}
{"type":"setup_toolchain","toolchain":{"version":"1.47.0","target":"x86_64-pc-windows-msvc"},"scope":"end"}
```

## Event: CheckMethod

**type:** check_method

**description:** Reports which method has been used to check whether a toolchain is compatible with a crate.

**fields:**

| name              | optional | condition                  | description                                |
|-------------------|----------|----------------------------|--------------------------------------------|
| toolchain         | no       |                            | The toolchain to be located or installed   |
| toolchain.version | no       |                            | The Rust version of the toolchain          |
| toolchain.target  | no       |                            | The target-triple of the toolchain         |
| method            | no       |                            | The method used to check for compatibility |
| method.type       | no       |                            | The type of method                         |
| method.args       | no       | method.type = `rustup_run` | The arguments provided to rustup           |
| method.path       | yes      | method.type = `rustup_run` | The path provided to rustup, if any        |


**example:**

```json lines
{"type":"check_method","toolchain":{"version":"1.37.0","target":"x86_64-pc-windows-msvc"},"method":{"rustup_run":{"args":["1.37.0-x86_64-pc-windows-msvc","cargo","check"],"path":"..\\air3\\"}}}
```

## Event: CheckResult

**type:** check_result

**description:** Reports the result of a `cargo-msrv` compatibility check.

**fields:**

| name              | optional | condition               | description                                           |
|-------------------|----------|-------------------------|-------------------------------------------------------|
| toolchain         | no       |                         | The toolchain to be located or installed              |
| toolchain.version | no       |                         | The Rust version of the toolchain                     |
| toolchain.target  | no       |                         | The target-triple of the toolchain                    |
| is_compatible     | no       |                         | Boolean value stating compatibility                   |
| error             | yes      | is_compatible = `false` | Error message of a failed compatibility check, if any |

**example:**

```json lines
{"type":"check_result","toolchain":{"version":"1.38.0","target":"x86_64-pc-windows-msvc"},"is_compatible":true}
```

```json lines
{"type":"check_result","toolchain":{"version":"1.37.0","target":"x86_64-pc-windows-msvc"},"is_compatible":false,"error":"error: failed to parse lock file at: .\\air3\\Cargo.lock\n\nCaused by:\ninvalid serialized PackageId for key `package.dependencies`\n"}
```

## Event: AuxiliaryOutput

**type:** auxiliary_output

**description:** Reports about additional output written by `cargo-msrv` when applicable. For example, if the
`--write-msrv` or `--write-toolchain-file` flag is provided, the MSRV will be written to the Cargo manifest or the
Rust toolchain file respectively. The act of writing this (additional) output is reported by this event.

**fields:**

| name             | optional | condition                       | description                                                                                      |
|------------------|----------|---------------------------------|--------------------------------------------------------------------------------------------------|
| destination      | no       |                                 | The destination of the auxiliary output                                                          |
| destination.type | no       |                                 | Type of destination, currently only "file"                                                       |
| destination.path | no       | if destination.type = `file`    | Path of the written or amended file                                                              |
| item             | no       |                                 | What kind of output is written                                                                   |
| item.type        | no       |                                 | Type of output item                                                                              |
| item.kind        | no       | if item.type = `msrv`           | To which field the MSRV was written in the Cargo manifest, "rust-version" or "metadata_fallback" |
| item.kind        | no       | if item.type = `toolchain_file` | Which toolchain file kind was written, "legacy" or "toml"                                        |


**example:**

```json lines
{"type":"auxiliary_output","destination":{"type":"file","path":"..\\air3\\Cargo.toml"},"item":{"type":"msrv","kind":"rust_version"}}
```

## Event: Progress

**type:** progress

**description:** Reports on the progress of an ongoing MSRV search. 

**fields:**

| name              | optional | condition | description                                                                                           |
|-------------------|----------|-----------|-------------------------------------------------------------------------------------------------------|
| current           | no       |           | Index of the currently running check into the sorted search space. Starts at `0`.                     |
| search_space_size | no       |           | The size of the search space.                                                                         |
| iteration         | no       |           | How many iterations have been completed, plus one for the currently running iteration. Starts at `1`. |

<!-- Future: add length of reduced set size -->


## Event: SubcommandInit

**type:** subcommand_init

**description:** Reports the start of a subcommand flow.

**fields:**

| name              | optional | condition | description                       |
|-------------------|----------|-----------|-----------------------------------|
| subcommand_id     | no       |           | A name identifying the subcommand |


## Event: SubcommandResult

**type:** subcommand_result

**description:** Reports the outcome of a subcommand flow.

**fields:**

| name                     | optional | condition                                                     | description                                                               |
|--------------------------|----------|---------------------------------------------------------------|---------------------------------------------------------------------------|
| subcommand_id            | no       |                                                               | A name identifying the subcommand                                         |
|                          |          |                                                               |                                                                           |
| result                   | no       | subcommand_id = `find`                                        | Result of find command                                                    |
| result.success           | no       | subcommand_id = `find`                                        | Whether the MSRV was found or not                                         |
| result.version           | no       | subcommand_id = `find` and result.success = `true`            | The Minimum Supported Rust Version (MSRV)                                 |
|                          |          |                                                               |                                                                           |
| result                   | no       | subcommand_id = `list`                                        | Result of list command                                                    |
| result.variant           | no       | subcommand_id = `list`                                        | Type of list output. Either `direct-deps` or `ordered-by-msrv`.           |
| result.list              | no       | subcommand_id = `list` and result.variant = `direct-deps`     | List of direct dependencies of the selected crate                         |
| result.list.name         | no       | subcommand_id = `list` and result.variant = `direct-deps`     | Name of the crate                                                         |
| result.list.version      | no       | subcommand_id = `list` and result.variant = `direct-deps`     | Version of the crate                                                      |
| result.list.msrv         | no       | subcommand_id = `list` and result.variant = `direct-deps`     | MSRV of the crate if any, `null` if the MSRV is not set                   |
| result.list.dependencies | no       | subcommand_id = `list` and result.variant = `direct-deps`     | Dependencies of the given crate, relevant for the MSRV                    |
| result.list              | no       | subcommand_id = `list` and result.variant = `ordered-by-msrv` | List of all dependencies relevant for the MSRV, categorised by their MSRV |
| result.list.msrv         | no       | subcommand_id = `list` and result.variant = `ordered-by-msrv` | A value for the MSRV specified by at least one crate                      |
| result.list.dependencies | no       | subcommand_id = `list` and result.variant = `ordered-by-msrv` | List of dependencies which specified the same value for the MSRV          |
|                          |          |                                                               |                                                                           |
| result                   | no       | subcommand_id = `set`                                         | Result of set command                                                     |
| result.version           | no       | subcommand_id = `set`                                         | Which version was set as MSRV                                             |
| result.manifest_path     | no       | subcommand_id = `set`                                         | Relative path of file where the MSRV was written to                       |
|                          |          |                                                               |                                                                           |
| result                   | no       | subcommand_id = `show`                                        | Result of show command                                                    |
| result.version           | no       | subcommand_id = `show`                                        | MSRV as set for the given crate                                           |
| result.manifest_path     | no       | subcommand_id = `show`                                        | Relative path of file where the MSRV was read from                        |
|                          |          |                                                               |                                                                           |
| result                   | no       | subcommand_id = `verify`                                      | Result of verify command                                                  ||
| result.toolchain         | no       | subcommand_id = `verify`                                      | The toolchain to be located or installed                                  |
| result.toolchain.version | no       | subcommand_id = `verify`                                      | The Rust version of the verified toolchain                                |
| result.toolchain.target  | no       | subcommand_id = `verify`                                      | The target-triple of the verified toolchain                               |
| result.is_compatible     | no       | subcommand_id = `verify`                                      | Boolean value stating compatibility                                       |
| result.error             | yes      | subcommand_id = `verify` and result.is_compatible = `false`   | Error message of a failed verify check, if any                            |

**example 1: find**

```json lines
{"type":"subcommand_result","subcommand_id":"find","result":{"version":"1.38.0","success":true}}
```

**example 2: list with direct-deps**

```json lines
{"type":"subcommand_result","subcommand_id":"list","result":{"variant":"direct-deps","list":[{"name":"crossbeam-channel","version": "0.5.4","msrv":"1.36.0","dependencies":["cfg-if","crossbeam-utils","num_cpus","rand","signal-hook"]}]}}
```

Formatted:

```json
{
  "type": "subcommand_result",
  "subcommand_id": "list",
  "result": {
    "variant": "direct-deps",
    "list": [
      {
        "name": "crossbeam-channel",
        "version": "0.5.4",
        "msrv": "1.36.0",
        "dependencies": [
          "cfg-if",
          "crossbeam-utils",
          "num_cpus",
          "rand",
          "signal-hook"
        ]
      }
    ]
  }
}
```

**example 3: list with ordered-by-msrv**

```json lines
{"type":"subcommand_result","subcommand_id":"list","result":{"variant":"ordered-by-msrv","list":[{"msrv":"1.38.0","dependencies":["storyteller"]},{"msrv":"1.36.0","dependencies":["crossbeam-channel","crossbeam-utils"]},{"msrv":null,"dependencies":["cfg-if","lazy_static"]}]}}
```

Formatted:

```json
{
  "type": "subcommand_result",
  "subcommand_id": "list",
  "result": {
    "variant": "ordered-by-msrv",
    "list": [
      {
        "msrv": "1.38.0",
        "dependencies": [
          "storyteller"
        ]
      },
      {
        "msrv": "1.36.0",
        "dependencies": [
          "crossbeam-channel",
          "crossbeam-utils"
        ]
      },
      {
        "msrv": null,
        "dependencies": [
          "cfg-if",
          "lazy_static"
        ]
      }
    ]
  }
}
```

**example 5: set**:

```json lines
{"type":"subcommand_result","subcommand_id":"set","result":{"version":"1.38.0","manifest_path":"..\\air3\\Cargo.toml"}}
```

**example 6: show**:

```json lines
{"type":"subcommand_result","subcommand_id":"show","result":{"version":"1.38.0","manifest_path":"..\\air3\\Cargo.toml"}}
```

**example 7: verify**:

```json lines
{"type":"subcommand_result","subcommand_id":"verify","result":{"toolchain":{"version":"1.38.0","target":"x86_64-pc-windows-msvc"},"is_compatible":true}}
```

## Event: TerminateWithFailure


# Output by subcommand

## \# cargo msrv (find)

## \# cargo msrv list

## \# cargo msrv set

## \# cargo msrv show

## \# cargo msrv verify