### ⏱️ Quick start

If all you want to do is find the MSRV for your package, you can run:

```shell
cargo msrv --bisect
```

This command will attempt to determine the MSRV by doing a binary search on
acceptable Rust releases. If you require additional options, please refer to the 
[`cargo-msrv commands`] section.

[`cargo-msrv commands`]: ../commands/index.md
