use crate::config::{CmdMatches, CmdMatchesBuilder};
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::{default_target, is_target_available};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub mod id {
    pub const SUB_COMMAND_MSRV: &str = "msrv";
    pub const ARG_SEEK_CMD: &str = "seek_cmd";
    pub const ARG_SEEK_PATH: &str = "seek_path";
    pub const ARG_SEEK_CUSTOM_TARGET: &str = "seek_target";
}

pub fn cli() -> App<'static, 'static> {
    App::new("cargo")
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
                .arg(
                    Arg::with_name(id::ARG_SEEK_PATH)
                        .long("path")
                        .help("Path to the cargo project directory.")
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
                ),
        )
}

pub fn cmd_matches(matches: &ArgMatches) -> TResult<CmdMatches> {
    let target = default_target()?;

    let seek = matches
        .subcommand_matches(id::SUB_COMMAND_MSRV)
        .ok_or(CargoMSRVError::UnableToParseCliArgs)?;

    let mut builder = CmdMatchesBuilder::new(&target);

    // set the command which will be used to check if a project can build
    let check_cmd = seek.value_of(id::ARG_SEEK_CMD);
    if let Some(cmd) = check_cmd {
        builder = builder.seek_cmd(cmd.to_string());
    }

    // set the cargo workspace path
    let crate_path = seek.value_of(id::ARG_SEEK_PATH);
    builder = builder.seek_path(crate_path);

    // set a custom target
    let custom_target = seek.value_of(id::ARG_SEEK_CUSTOM_TARGET);
    if let Some(target) = custom_target {
        builder = builder.target(target);
    }

    Ok(builder.build())
}
