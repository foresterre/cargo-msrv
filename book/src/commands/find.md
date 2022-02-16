# cargo-msrv

# COMMAND

* Standalone: `cargo-msrv [options]`
* Through Cargo: `cargo msrv [options]`

## DESCRIPTION

Find the MSRV for your project.

This command will test your project by running various Rust toolchains against your project. The order in which these
toolchains will be tested, and the amount of tests ran, depends on the search strategy, the amount of toolchains
available and of course the limiting factor of the project which will determine the MSRV. We usually call each test a
cargo-msrv _check_. By default, the check command, the command used to test whether toolchain passes or fails a check,
is `cargo check --all`.

There are currently two search strategies: _linear_ (default) and _bisect_. Linear tests projects against toolchains in a
most-recent to least-recent order. When a check fails, the previous Rust (if any) version is returned as the MSRV (i.e. the highest still
toolchain for which a check command passes). Bisect tests projects using a binary search. This can be significantly faster,
so it's usually advisable to enable it by default. 

### Why run against complete toolchains?

Running against a complete toolchain may seem like a lot of wasted computing power. Why not run against just the AST, and
(conditionally) tag each AST node with a supported from version (or query it, as library functions already have an 'available
from' Rust version)? 

Earlier we developed a prototype to do exactly this, and we may still add it as an optional strategy in the future, however
we found that the selection of the MSRV of a toolchain is not just limited by the source code itself. External factors
such as Rust editions or knobs in the Cargo manifest also impact the MSRV for a crate. As such, the running a complete
toolchain helps us to be more precise<sup>1</sup>.

### Future work

_1. Currently, the 'Find your MSRV' action is defined at as the top-level `cargo-msrv` command. We hope to move it to its own subcommand
in the near future, e.g. `cargo-msrv find` (subcommand name subject to change)._

_2. We want to eventually add a combination-of-strategies strategy which can combine result of other strategies to come
to a possibly more precise definition._

_3. If you come up with a strategy which will add value to cargo-msrv, feel free to contribute the idea, or even an
implementation. If you don't know where to start, create a new issue, we're happy to help!_ 

## OPTIONS

**`--bisect`**

Use a binary search to find the MSRV. This is usually faster than using a linear search.
The binary search strategy is the default since `cargo-msrv v0.14.0`.

**`--linear`**

Use a linear search to find the MSRV, by checking toolchains from latest to earliest.
The linear search strategy was the default prior to `cargo-msrv v0.14.0`.

**`-h, --help`**

Prints help information

**`--include-all-patch-releases`**

Include all patch releases, instead of only the last. By default, after the list of Rust releases has been fetched, we\
only keep the highest minor version for each Rust release. Say the list of Rust releases would be `["1.31.1", "1.31.0", "1.30.0]`,
then we discard Rust `1.31.0`, as you would usually not depend on the non-bugfixed compiler releases, and the patch version
does not contain new features, thus no features to impact the MSRV. When you provide this flag however, these additional
patch versions will be included in the search space.



**`--ignore-lockfile`**

Temporarily (re)moves the lockfile, so it will not interfere with the building process. This is important when
testing against Rust versions prior to 1.38.0, for which Cargo does not recognize the new v2 lockfile (`Cargo.lock`),
or some crates which use the even newer v3 lockfile. 

**`--log-level` level**

Specify the severity of debug logs which the program will write to the log output.
Possible values are: `error`, `warn`, `info` (default), `debug` and `trace`. 
Lower severities include messages of higher severities.
When `--no-log` is present, this option will be ignored.


**`--log-target` log_target**

Specify where cargo-msrv should output its internal debug logs.
Possible values are `file` (default) and `stdout`.
The log output of `stdout` may interfere with user output. We would suggest to use `--no-user-output` in tandem
with `--log-target stdout`. When `--no-log` is present, this option will be ignored.


**`--max` version**

Latest (most recent) version to take into account. The version must match a valid three component Rust toolchain version, 
and be semver compatible. An example of an acceptable versions is "1.35.0", while "1.35", "^1.35.0" and "1.35.0-beta" are not valid.


**`--min` version**

Earliest (least recent) version to take into account. The version must match a valid three component Rust toolchain version,
and be semver compatible. Edition aliases may also be used. An example of an acceptable versions is "1.35.0", while
"1.35", "^1.35.0" and "1.35.0-beta" are not valid. Editions map to the first version in which they were introduced, so
for example "1.56.0" for edition "2021".

**`--no-check-feedback`**

If provided, the outcome of individual checks will not be printed. These prints provide feedback, about the order in which
checks ran, and their results. This is especially useful if you want to know why a certain Rust version was deemed to be
incompatible, for example, so you can identify Rust features which require a certain minimum Rust version.  

**`--no-log`**

Do not write (internal) debug log output to the log target.


**`--no-read-min-edition`**
            
If provided, the 'package.edition' value in the Cargo.toml will not be used to reduce search space.
By default, the edition is read from the `Cargo.toml` file and used as the minimum Rust version. See also `--min`.


**`--no-user-output`**

Disables printing of diagnostic status messages. Useful when internal log output messages are printed to the stdout,
using `--log-target stdout`, so no clipping between the user output prints and log message prints will take place.
When present, the `--output-format [value]` option will be ignored.

**`--output-format` format**

Output diagnostic status messages in machine-readable format. Machine-readable status updates will be printed in the
requested format to stdout. The only accepted format is currently "json", which will print diagnostic messages in a JSON
format. When this option is absent, human-readable output will be printed. Diagnostic messages can be disabled entirely
using the `--no-user-output` flag.

**`--release-source` source**

Select the rust-releases source to use as the release index. Available options are `rust-changelog` and `rust-dist`.
The first will parse the Rust changelog file to determine which Rust releases have been made, while the second will index
the Rust S3 distribution bucket.

**`--path` directory-path**

Path to the cargo project directory. This directory should contain a Cargo manifest (i.e. `Cargo.toml`) file. The given
path should end in the Cargo manifest file. A valid path would be `/home/user/project`. A path like `/home/user/project/Cargo.toml`
is incorrect.

**`--target` target**

Supply a custom target triplet to use as Rust distribution. If absent, the rustup default toolchain is used.

**`--toolchain-file`**

Output a rust-toolchain file with the determined MSRV as toolchain. The toolchain file will pin the Rust version for this crate. 
See [here](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file) for more about the toolchain-file.

**`-V, --version`**

Prints cargo-msrv version information

**`--verify` (DEPRECATED)**

Verify the MSRV specified by a crate author in the 'package.rust-version' or the 'package.metadata.msrv' key, in the
Cargo manifest `Cargo.toml`. When this flag is present, cargo-msrv will not attempt to determine the true MSRV. 
Instead, it only attempts to verify whether for the specified MSRV, the _cargo-msrv check_ command passes.

_DEPRECATED: use `cargo msrv verify` instead_

**`--` ...cmd** 

When provided, the trailing command (`cmd`) will be used as the _cargo-msrv check_ command, instead of the default
`cargo check --all`. This `cmd` must be runnable by `rustup` through `rustup run <toolchain> <cmd>`.


## EXAMPLES

1. Try to determine the MSRV for the crate in your current working directory, using the binary search strategy.

```shell
cargo msrv --bisect
```

or (from cargo-msrv `v0.14.0`, `bisect` is the default search method):

```shell
cargo msrv
```

2. Try to determine the MSRV for the crate in your current working directory, using the linear search strategy.

```shell
cargo msrv --linear
```

NB: Prior to cargo-msrv `v0.14.0`, `linear` was the default search strategy, and no flag was available explicitly
use this search strategy.

3. Try to determine the MSRV for the crate in your current working directory, using a custom cargo-msrv check command:
`cargo test`.

```shell
cargo msrv -- cargo test
```

4. Try to determine the MSRV for the crate in your current working directory, but use the JSON machine-readable output
format.

```shell
cargo msrv --output-format json
```

## FOOTNOTES

<sup>1</sup> Precision is of course a debatable concept. In this case we note that "a toolchain must be able
to pass the cargo-msrv check command for a crate" (in its broadest sense).
