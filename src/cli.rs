use crate::config::{CmdMatches, CmdMatchesBuilder};
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::default_target;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub mod id {
    pub const SUB_COMMAND_MSRV: &str = "msrv";
    pub const ARG_SEEK_CMD: &str = "seek_cmd";
    pub const ARG_SEEK_PATH: &str = "seek_path";
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
                ),
        )
}

pub fn cmd_matches(matches: &ArgMatches) -> TResult<CmdMatches> {
    let target = default_target()?;

    let seek = matches
        .subcommand_matches(id::SUB_COMMAND_MSRV)
        .ok_or(CargoMSRVError::UnableToParseCliArgs)?;

    let seek_cmd = seek.value_of(id::ARG_SEEK_CMD);
    let seek_path = seek.value_of(id::ARG_SEEK_PATH);

    let mut builder = CmdMatchesBuilder::new(&target);

    if seek_cmd.is_some() {
        builder = builder.seek_cmd(seek_cmd.unwrap().to_string());
    }

    builder = builder.seek_path(seek_path);

    Ok(builder.build())
}
