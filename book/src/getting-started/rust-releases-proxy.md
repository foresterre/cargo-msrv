### Rust Releases: HTTP PROXY

`cargo-msrv` depends on the [rust-releases](https://github.com/foresterre/rust-releases/) crate to determine which Rust versions exist. This is a necessary evil for
 the `cargo msrv find` and `cargo msrv verify` subcommands.
 
To fetch an index of known Rust releases, it accesses the network. By default, the Rust GitHub repository is used to determine
which stable releases and toolchains are available. As an alternative, this data can also be fetched from the Rust AWS S3
distribution bucket.

The source can be set with the `--release-source <source>` flag. The possible values are respectively `rust-changelog` and `rust-dist`,
for the Rust GitHub repository and the Rust AWS S3 distribution bucket. For example: `cargo msrv find --release-source rust-changelog`.



#### Release source: `rust-changelog`

[rust-releases](https://github.com/foresterre/rust-releases/) uses [ureq](https://crates.io/crates/ureq) as HTTP client
for the `rust-changelog` source. From `cargo-msrv 0.17.1` (and [rust-releases 0.29.0](https://github.com/foresterre/rust-releases/releases/tag/v0.29.0)
respectively), `ureq` has been configured to support configuring a network proxy from the environment.

The environment variable, `ureq` uses [are](https://docs.rs/ureq/2.11.0/src/ureq/proxy.rs.html#87-92):
 
- `ALL_PROXY` or `all_proxy` or,
- `HTTPS_PROXY` or `https_proxy` or,
- `HTTP_PROXY` or `http_proxy`

The environment variable can be configured as follows:

`<protocol>://<user>:<password>@<host>:port`, where all parts except host are optional.

The `<protocol>` must be one of: `http` (`socks4`, `socks4a` and `socks5` are currently not enabled). The default is `http`.

The default `<port>` is 80 when the `<protocol>` is `http` .

Examples: 
- `localhost`
- `http://127.0.0.1:8080`


#### Release source: `rust-dist`

TODO: Not configured specifically by cargo-msrv, but could be the case.

The following crates are used for the `rust-dist` source:

- [aws-config](https://crates.io/crates/aws-config)
- [aws-sdk-s3](https://crates.io/crates/aws-sdk-s3)

Probably also relevant as transitive dependencies are:

- [aws-smithy-http](https://crates.io/crates/aws-smithy-http)
