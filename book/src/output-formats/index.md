# Output formats

In `cargo-msrv` we to status of the program is reported via events. These events are issued at several stages of the
program execution. As a user of `cargo-msrv`, you may choose how these events are formatted into a human-readable
or machine-readable representation. User output may also be disabled altogether.

The next section gives an overview of the supported representations. Thereafter, you may find an index to
the chapters which detail each representation.

## Choosing an output format

The first output format is the `human` output format. As the name suggests,
its output is meant to be interpreted by humans, and consists of elaborate colouring and
custom styling.

The second output format is the `json` output format. This is a detailed machine-readable
format and also the format which most closely mimics the events as they are reported internally.
Events are printed in a `json-lines` (`jsonl`) format to *stderr*.

The third output-format is the `minimal` output format. This format is intended to be used by shell scripts
or programs which do not require detailed output. Its format does not require complex parsing, and only
reports the final results of commands.

The fourth option is to not print any user output. This is uncommon, but may be used in conjunction with
printing debug (i.e. developer) output only, so the debug output is not overwritten by the user output.

## The output formats

* [human](human.md) (default)
* [json](json.md)
* [minimal](minimal.md)
* [no-user-output](no-user-output.md)