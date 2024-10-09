# cargo-msrv list

# COMMAND

* Standalone: `cargo-msrv list [options]`
* Through Cargo: `cargo msrv list [options]`

# PREVIEW

[![asciicast](https://asciinema.org/a/679857.svg)](https://asciinema.org/a/679857)

# DESCRIPTION

List the author specified MSRV for each depended-upon package.

Authors may specify the MSRV for their crate by adding the `package.rust-version` key to the `Cargo.toml` manifest.
See the [Cargo book](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field) for more. This
value is supported from Rust 1.56 onwards.

```toml
[package]
rust-version = "1.56"
```

For crates which have an MSRV prior to Rust 1.56, you can use the `package.metadata.msrv` key in the `Cargo.toml`
manifest
instead. The `package.metadata` table exists specifically for tools like cargo-msrv, and within this table,
Cargo will not warn about keys it does not understand. Note that the use of this key is tailored to cargo-msrv and may
not be supported by other tools.

```toml
[package.metadata]
msrv = "1.53.0"
```

Both `package.rust-version` and `package.metadata.msrv` require a two or three component version number, without semver
operators
or pre-release identifiers. For example, `1.56` and `1.56.0` are both valid, while `^1.56.0` and `1.56.0-beta` are not.

# OPTIONS

**`--variant` variant**

Type of table to print.

The `variant` must be one of: `ordered-by-msrv` (default) or `direct-deps`.

When the `variant` is `ordered-by-msrv`, the program will print a table which lists the MSRV for both
direct and transitive dependencies. The table is sorted by MSRV. When a crate author did not specify an MSRV yet, the
cell in the MSRV row will be empty.

When the `variant` is `direct-deps`, the program will print a table which lists the following properties for each
direct-dependency of the given crate: the name of the dependency, the version of the dependency, the MSRV (empty if not
specified), it's dependencies.

# EXAMPLES

1. List the MSRV's for both direct and transitive dependencies, grouped by MSRV.

```shell
cargo msrv list
```

Output for cargo-msrv commit c76b45a7ae39b52294e303eca6da56fda45b3feb:

```text
Fetching index
┌────────┬───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ MSRV   ┆ Dependency                                                                                                                                │
╞════════╪═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╡
│        ┆ typed-builder, tracing-subscriber, tracing-appender, tracing, toml_edit, petgraph, once_cell, json, indicatif, dirs, console,             │
│        ┆ comfy-table, clap, cargo_metadata, tracing-serde, tracing-log, tracing-core, thread_local, smallvec, sharded-slab, serde_json, serde,     │
│        ┆ ansi_term, time, crossbeam-channel, tracing-attributes, pin-project-lite, cfg-if, kstring, itertools, indexmap, combine,                  │
│        ┆ rust-releases-rust-dist, rust-releases-rust-changelog, rust-releases-core, fixedbitset, regex, number_prefix, lazy_static, dirs-sys,      │
│        ┆ winapi, unicode-width, terminal_size, libc, encode_unicode, strum_macros, strum, crossterm, vec_map, textwrap, strsim, bitflags, atty,    │
│        ┆ ansi_term, semver, cargo-platform, camino, unicode-xid, log, ryu, itoa, serde_derive, crossbeam-utils, either, hashbrown, autocfg,        │
│        ┆ memchr, bytes, tokio, rust-releases-io, rusoto_s3, rusoto_core, chrono, regex-syntax, aho-corasick, redox_users,                          │
│        ┆ winapi-x86_64-pc-windows-gnu, winapi-i686-pc-windows-gnu, heck, signal-hook-mio, signal-hook, parking_lot, mio, crossterm_winapi,         │
│        ┆ hermit-abi, tokio-macros, signal-hook-registry, num_cpus, directories-next, attohttpc, xml-rs, futures, async-trait, rustc_version,       │
│        ┆ rusoto_signature, rusoto_credential, hyper-tls, hyper, http, crc32fast, base64, time, num-traits, num-integer, redox_syscall, getrandom,  │
│        ┆ unicode-segmentation, parking_lot_core, lock_api, instant, ntapi, miow, dirs-sys-next, wildmatch, url, openssl, native-tls, flate2,       │
│        ┆ futures-util, futures-task, futures-sink, futures-io, futures-executor, futures-core, futures-channel, semver, time, sha2,                │
│        ┆ percent-encoding, md5, hmac, hex, zeroize, shlex, dirs-next, tokio-native-tls, want, tower-service, socket2, httpdate, httparse,          │
│        ┆ http-body, fnv, wasi, scopeguard, matches, idna, form_urlencoded, openssl-sys, foreign-types, tempfile, security-framework-sys,           │
│        ┆ security-framework, schannel, openssl-probe, miniz_oxide, slab, proc-macro-nested, proc-macro-hack, pin-utils, futures-macro,             │
│        ┆ semver-parser, version_check, time-macros, stdweb, standback, const_fn, opaque-debug, digest, cpufeatures, block-buffer, crypto-mac,      │
│        ┆ try-lock, unicode-normalization, unicode-bidi, vcpkg, pkg-config, cc, foreign-types-shared, remove_dir_all, rand, core-foundation-sys,    │
│        ┆ core-foundation, adler, time-macros-impl, wasm-bindgen, stdweb-internal-runtime, stdweb-internal-macros, stdweb-derive, discard,          │
│        ┆ generic-array, subtle, tinyvec, rand_hc, rand_core, rand_chacha, wasm-bindgen-macro, sha1, base-x, typenum, tinyvec_macros, ppv-lite86,   │
│        ┆ wasm-bindgen-macro-support, wasm-bindgen-shared, wasm-bindgen-backend, bumpalo                                                            │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 1.31.0 ┆ syn, quote, proc-macro2, thiserror, thiserror-impl                                                                                        │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 1.51.0 ┆ rust-releases                                                                                                                             │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 1.53.0 ┆ cargo-msrv                                                                                                                                │
└────────┴───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

```

NB: The dependencies which are listed with an empty MSRV cell do not specify a MSRV yet. At the time of writing, most
dependencies in the cargo-msrv dependency tree did not have an MSRV defined.

2. List the MSRV's for your direct dependencies using

```shell
cargo msrv list --variant direct-deps
```

Output for cargo-msrv commit c76b45a7ae39b52294e303eca6da56fda45b3feb:

```text
Fetching index
┌────────────────────┬─────────┬────────┬──────────────────────────────────────────────────────────────────────────────┐
│ Dependency         ┆ Version ┆ MSRV   ┆ Depends on                                                                   │
╞════════════════════╪═════════╪════════╪══════════════════════════════════════════════════════════════════════════════╡
│ typed-builder      ┆ 0.9.1   ┆        ┆ proc-macro2, quote, syn                                                      │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ tracing-subscriber ┆ 0.3.1   ┆        ┆ ansi_term, lazy_static, matchers, parking_lot, regex, serde, serde_json,     │
│                    ┆         ┆        ┆ sharded-slab, smallvec, thread_local, time, tracing, tracing-core,           │
│                    ┆         ┆        ┆ tracing-log, tracing-serde, criterion, log, regex, time, tokio, tracing,     │
│                    ┆         ┆        ┆ tracing-futures, tracing-log                                                 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ tracing-appender   ┆ 0.2.0   ┆        ┆ crossbeam-channel, time, tracing-subscriber, tempfile, time, tracing         │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ tracing            ┆ 0.1.29  ┆        ┆ cfg-if, log, pin-project-lite, tracing-attributes, tracing-core, criterion,  │
│                    ┆         ┆        ┆ futures, log, tokio, wasm-bindgen-test                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ toml_edit          ┆ 0.8.0   ┆        ┆ combine, indexmap, itertools, kstring, serde, criterion, pretty_assertions,  │
│                    ┆         ┆        ┆ serde_json, toml, toml-test-harness                                          │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ rust-releases      ┆ 0.16.1  ┆ 1.51.0 ┆ rust-releases-channel-manifests, rust-releases-core, rust-releases-io,       │
│                    ┆         ┆        ┆ rust-releases-rust-changelog, rust-releases-rust-dist,                       │
│                    ┆         ┆        ┆ rust-releases-rust-dist-with-cli, yare                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ petgraph           ┆ 0.6.0   ┆        ┆ fixedbitset, indexmap, quickcheck, serde, serde_derive, bincode, defmac,     │
│                    ┆         ┆        ┆ itertools, odds, rand                                                        │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ once_cell          ┆ 1.8.0   ┆        ┆ parking_lot, crossbeam-utils, lazy_static, regex                             │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ json               ┆ 0.12.4  ┆        ┆                                                                              │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ indicatif          ┆ 0.16.2  ┆        ┆ console, lazy_static, number_prefix, rayon, regex, unicode-segmentation,     │
│                    ┆         ┆        ┆ unicode-width, rand, tokio                                                   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ dirs               ┆ 4.0.0   ┆        ┆ dirs-sys                                                                     │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ console            ┆ 0.15.0  ┆        ┆ libc, once_cell, regex, terminal_size, unicode-width, encode_unicode,        │
│                    ┆         ┆        ┆ winapi, winapi-util                                                          │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ comfy-table        ┆ 4.1.1   ┆        ┆ crossterm, strum, strum_macros, unicode-width, criterion, doc-comment,       │
│                    ┆         ┆        ┆ pretty_assertions, proptest                                                  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ clap               ┆ 2.33.3  ┆        ┆ atty, bitflags, clippy, strsim, term_size, textwrap, unicode-width, vec_map, │
│                    ┆         ┆        ┆ yaml-rust, lazy_static, regex, version-sync, ansi_term                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ cargo_metadata     ┆ 0.14.1  ┆        ┆ camino, cargo-platform, derive_builder, semver, serde, serde_json            │
└────────────────────┴─────────┴────────┴──────────────────────────────────────────────────────────────────────────────┘
```

NB: The dependencies which are listed with an empty MSRV cell do not specify a MSRV yet. At the time of writing, most
dependencies in the cargo-msrv dependency tree did not have an MSRV defined.
