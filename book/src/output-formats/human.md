# Output format: human

This is the default output format. It can also be specified using the `--output-format human` option.

The output of the 'human' output format is intended to be interpreted by humans. It uses colour and custom printed
layouts to convey its information to the user. 

In the next section, examples are given for each subcommand and a specific use case. You may run `cargo msrv help` to
review all flags and options available.

# Output by subcommand

## \# cargo msrv (find)

**I want to find the MSRV of my project**

Use: `cargo msrv`

[![Screencast: find the MSRV](https://asciinema.org/a/530521.svg)](https://asciinema.org/a/530521)

The output shows for each checked toolchain whether it is determined to be compatible or not.
If a toolchain is not compatible, a reason is printed which may help you discover why it is not deemed compatible.

cargo-msrv will show a summary after the search completes. The summary consists of the search space considered,
the search method used, the compiler target and of course the MSRV.

It is also possible that no MSRV could be found, for example if the program is not valid Rust code (i.e. would not compile).

**I want to find the MSRV and write the result to the Cargo manifest**

Use the `--write-msrv` flag: `cargo msrv --write-msrv`

[![Screencast: find the MSRV and write the result to the Cargo manifest](https://asciinema.org/a/530521.svg)](https://asciinema.org/a/530521)

The output is the same as for `cargo msrv`, plus an additional message which states that the MSRV has been written to
the Cargo manifest.

_Support for also writing to the clippy config is tracked in 
[issue 529](https://github.com/foresterre/cargo-msrv/issues/529)_.

**I want to find the MSRV and limit or increase the search space**

Use the `--min` and/or `--max` options: `cargo msrv --min <Rust version or edition> --max <Rust version>`

[![Sceencast: find the MSRV with a customized search space](https://asciinema.org/a/SEqHCRxI5xe0eizaBbIraHZcV.svg)](https://asciinema.org/a/SEqHCRxI5xe0eizaBbIraHZcV)

By default, the search space is limited by the edition specified in the Cargo manifest. You may use the above
options to override the limits of the search space. The output will be the same as otherwise running `cargo msrv`.

In the example we specified the minimal version by specifying a Rust edition. We also could've specified a Rust version
instead, e.g. `1.10` or `1.20.0`. It is not possible for the maximum considered version to specify an edition.

**I want to find the MSRV, but use a linear search**

Use the `--linear` flag: `cargo msrv --linear`

[![Screencast: find the MSRV using a linear search](https://asciinema.org/a/530645.svg)](https://asciinema.org/a/530645)

We use the bisection search method to speed up the search for the MSRV considerably, but sometimes a linear search
can be useful, for example if the search space is very small. The output will be the same as otherwise running
`cargo msrv`, except of course for the order in which the search is performed.

## \# cargo msrv list

**I want to list the MSRV's of all dependencies**

Use: `cargo msrv list`

[![Screencast: list MSRV's of dependencies](https://asciinema.org/a/530652.svg)](https://asciinema.org/a/530652)

This example shows how to list the MSRV's of dependencies. The MSRV's are sourced from their Cargo manifests.

**I want to list the MSRV's of my direct dependencies**

Use the `--variant` option: `cargo msrv list --variant direct-deps`

[![Screencast: list MSRV's of direct dependencies](https://asciinema.org/a/AU2Xaq1hrXUYfjLdUvDzZHaCC.svg)](https://asciinema.org/a/AU2Xaq1hrXUYfjLdUvDzZHaCC)

In this example, we instead list the MSRV's of the dependencies specified in the Cargo manifest.

## \# cargo msrv set

## \# cargo msrv show

## \# cargo msrv verify

