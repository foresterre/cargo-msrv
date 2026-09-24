🚧 Section is work-in-progress.

# 🌱 Concepts

## Rust Releases Index

### Release Source

`cargo-msrv` requires a list of available Rust releases to determine or verify the MSRV. It can fetch this list
of releases from several places (which each vary in completeness, correctness, and size/fetch time). 

The following list are all release sources which can be configured (e.g. `cargo msrv find --release-source <value>`).
`cargo-msrv` can be compiled with or without these release sources (our own release includes all), except for `bundled`
 which is always included.

* rust-changelog (default)
* bundled-unless-outdated
* github
* rust-dist
* bundled

#### rust-changelog

Fetches the `RELEASES.md` changelog from the official [rust](https://github.com/rust-lang/rust/blob/main/RELEASES.md) GitHub
repository. Does a time check which compares the callers computer date against the latest release date, because the changelog
may be slightly ahead when the Rust developers are cutting a release.

This source only provides version numbers and release dates, so toolchains etc. are not included.

This source is the default because it is a single file (unlike say `github` releases which needs to consider pagination
and adhere to the max API limits). It is very fast, consistently up to date and it is reasonably easy to parse.

#### bundled-unless-outdated

Same as `bundled`, except that if its configured max age (default is 7 days) has been reached since it was generated, then
it falls back to the (configured) fallback release source, which is the same list of release sources (defaults follow the
same ordering rules). 

Note that the bundled source is complete (it's effectively the `rust-dist` source packaged in a much smaller format, and
pre-packaged as Rust source code for easy embedding), up to its generation day, and includes not just versions but also
full toolchain and target information. 

If it is outdated and another source is fetched to amend the bundled source, that source may be much less complete which
can be a source of confusion if you're combining it with strict filters on e.g. the host target (which the online fetched
release source may not have). In general `cargo-msrv` tries to have best effort defaults. If you find a case where it is
not entirely the case, open an issue (be sure to include your use case and suggested best effort!).

#### github

Uses the GitHub releases [list](https://github.com/rust-lang/rust/releases) via the GitHub API to determine the available
versions. It takes the release date from the API's release date. NB: For versions prior to Rust 1.47.0, this release date
is incorrect (for versions `<= 1.46.0` releases were created in one big batch on 2020-09-10). `cargo-msrv` barely uses the
release date so this isn't a problem.

#### rust-dist

The most complete release source. All available Rust release and distribution information can be found here in one place,
if you're willing to fetch and parse it all. Fetching it takes a long time because it needs to go through a lot of information,
much of which is not relevant if you only care about releases (and not everything, all distributions ever created contain).

Usually version information is enough and then `rust-dist`'s added toolchain data is unnecessary, but if you do stuff like
cross-compilation, you may want to actually select toolchains which are really available, iso guessing that they are.
This is the place where the nitty gritty gray areas between the differences of "releases" vs "distributions" start to fade,
and where a vague general term like "MSRV" (minimum supported Rust version) starts to require a much more guarded definition
wrt what "Rust" then means (e.g. is it only the language compatibility, or do the underlying platforms matter?).

The `bundled` source is generated from `rust-dist` by the [rust-releases' bundled generator](https://github.com/foresterre/rust-releases/tree/7d3ef7618b53ead14c2945ec8501b9940393a780/crates/rust-releases-bundled-generator)

### bundled

Same as `rust-dist`, but generated and embedded as generated Rust code at a certain point in time. Only as up to date
as the time where (a) `rust-releases-bundled` was generated and releases, (b) `cargo-msrv` updated its lockfile to include
the latest `rust-releases-bundled` and (c) `cargo-msrv` was released.

In general ever new `cargo-msrv` release should contain the latest generated set, available on or prior to `cargo-msrv's`
release date. If it didn't, please open an issue.

## Resolver

* run-toolchain resolver (default): resolver which runs actual toolchains against a crate
* rust-version resolver: author defined resolver, used by `cargo-msrv list`
