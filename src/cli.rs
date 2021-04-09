use crate::config::{CmdMatches, CmdMatchesBuilder};
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::{default_target, is_target_available};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub mod id {
    pub const SUB_COMMAND_MSRV: &str = "msrv";
    pub const ARG_SEEK_PATH: &str = "seek_path";
    pub const ARG_SEEK_CUSTOM_TARGET: &str = "seek_target";
    pub const ARG_CUSTOM_CHECK: &str = "custom_check";
    pub const ARG_INCLUDE_ALL_PATCH_RELEASES: &str = "include_all_patch";
    pub const ARG_MIN: &str = "min";
    pub const ARG_MAX: &str = "max";
    pub const ARG_BISECT: &str = "bisect";
    pub const ARG_TOOLCHAIN_FILE: &str = "toolchain_file";
}

pub fn cli() -> App<'static, 'static> {
    App::new("cargo-msrv")
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Martijn Gribnau <garm@ilumeo.com>")
        .global_setting(AppSettings::NextLineHelp)
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .max_term_width(120)
        .subcommand(
            SubCommand::with_name(id::SUB_COMMAND_MSRV)
                .usage("cargo msrv [OPTIONS]")
                .about("Helps with finding the Minimal Supported Rust Version (MSRV)")
                .after_help("\
If arguments are provided after two dashes (`--`), they will be used as a custom command to validate \
whether a Rust version is compatible. By default for this validation the command `cargo build` is \
used. Commands should be runnable by rustup, i.e. validation commands will be passed to rustup like \
so: `rustup run <toolchain> <COMMAND...>`. You'll only need to provide the <COMMAND...> part.")
                .arg(
                    Arg::with_name(id::ARG_SEEK_PATH)
                        .long("path")
                        .help("Path to the cargo project directory")
                        .takes_value(true)
                        .value_name("DIR")
                        .validator(|value| {
                            std::fs::metadata(&value)
                                .map_err(|_| "Path doesn't exist.".to_string())
                                .and_then(|m| {
                                    if m.is_dir() {
                                        Ok(())
                                    } else {
                                        Err("Not a directory.".to_string())
                                    }
                                })
                        }),
                )
                .arg(
                    Arg::with_name(id::ARG_SEEK_CUSTOM_TARGET)
                        .long("target")
                        .help("Check against a custom target (instead of the rustup default)")
                        .takes_value(true)
                        .value_name("TARGET")
                        .validator(|value| {
                            is_target_available(&value).map_err(|_| {
                                "The provided target is not available. Use `rustup target list` to \
                                 review the available targets."
                                    .to_string()
                            })
                        }),
                )
                .arg(Arg::with_name(id::ARG_INCLUDE_ALL_PATCH_RELEASES)
                    .long("include-all-patch-releases")
                    .help("Include all patch releases, instead of only the last")
                    .takes_value(false)
                )
                .arg(Arg::with_name(id::ARG_MIN)
                    .long("minimum")
                    .help("Earliest version to take into account")
                    .long_help("Defaults to 1.31.0 which is the first Rust 2018 compatible version")
                    .takes_value(true)
                    .default_value("1.31.0")
                )
                .arg(Arg::with_name(id::ARG_MAX)
                    .long("maximum")
                    .help("Latest version to take into account")
                    .takes_value(true)
                )
                .arg(Arg::with_name(id::ARG_BISECT)
                    .long("bisect")
                    .help("Use a binary search to find the MSRV instead of a linear search")
                    .takes_value(false)
                )
                .arg(Arg::with_name(id::ARG_TOOLCHAIN_FILE)
                    .long("toolchain-file")
                    .help("Output a rust-toolchain file with the MSRV as toolchain")
                    .long_help("Output a rust-toolchain file with the MSRV as toolchain. \
                    The toolchain file will pin the Rust version for this crate. \
                    See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more")
                )
                .arg(
                    Arg::with_name(id::ARG_CUSTOM_CHECK)
                        .value_name("COMMAND")
                        .help("If given, this command is used to validate if a Rust version is \
                        compatible. Should be available to rustup, i.e. the command should work like \
                        so: `rustup run <toolchain> <COMMAND>`. \
                        The default check action is `cargo check --all`")
                        .multiple(true)
                        .last(true)

                )
        )
}

pub fn cmd_matches<'a>(matches: &'a ArgMatches<'a>) -> TResult<CmdMatches<'a>> {
    let target = default_target()?;

    let arg_matches = matches
        .subcommand_matches(id::SUB_COMMAND_MSRV)
        .ok_or(CargoMSRVError::UnableToParseCliArgs)?;

    let mut builder = CmdMatchesBuilder::new(&target);

    // set the command which will be used to check if a project can build
    let check_cmd = arg_matches.values_of(id::ARG_CUSTOM_CHECK);
    if let Some(cmd) = check_cmd {
        builder = builder.check_command(cmd.collect());
    }

    // set the cargo workspace path
    let crate_path = arg_matches.value_of(id::ARG_SEEK_PATH);
    builder = builder.crate_path(crate_path);

    // set a custom target
    let custom_target = arg_matches.value_of(id::ARG_SEEK_CUSTOM_TARGET);
    if let Some(target) = custom_target {
        builder = builder.target(target);
    }

    if let Some(min) = arg_matches.value_of(id::ARG_MIN) {
        builder = builder.minimum_version(Some(rust_releases::semver::Version::parse(min)?))
    }

    if let Some(max) = arg_matches.value_of(id::ARG_MAX) {
        builder = builder.maximum_version(Some(rust_releases::semver::Version::parse(max)?))
    }

    builder = builder.bisect(arg_matches.is_present(id::ARG_BISECT));

    builder = builder
        .include_all_patch_releases(arg_matches.is_present(id::ARG_INCLUDE_ALL_PATCH_RELEASES));

    builder = builder.output_toolchain_file(arg_matches.is_present(id::ARG_TOOLCHAIN_FILE));

    Ok(builder.build())
}
