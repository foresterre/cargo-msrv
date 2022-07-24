use crate::config::{OutputFormat, TracingTargetOption};

use crate::log_level::LogLevel;
use clap::AppSettings;
use clap::ArgGroup;
use clap::Args;
use std::path::PathBuf;

// Cli Options shared between subcommands
#[derive(Debug, Args)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
#[clap(group(ArgGroup::new("paths").args(&["path", "manifest-path"])))]
pub struct SharedOpts {
    /// Path to cargo project directory
    #[clap(long, value_name = "Crate Directory", global = true)]
    pub path: Option<PathBuf>,

    /// Path to cargo manifest file
    #[clap(long, value_name = "Cargo Manifest", global = true)]
    pub manifest_path: Option<PathBuf>,

    #[clap(flatten)]
    pub user_output_opts: UserOutputOpts,

    #[clap(flatten)]
    pub debug_output_opts: DebugOutputOpts,
}

#[derive(Debug, Args)]
#[clap(next_help_heading = "USER OUTPUT OPTIONS", setting = AppSettings::DeriveDisplayOrder)]
pub struct UserOutputOpts {
    /// Set the format of user output
    #[clap(long,
        possible_values = OutputFormat::custom_formats(),
        default_value_t,
        value_name = "FORMAT",
        global = true,
    )]
    pub output_format: OutputFormat,

    /// Disable user output
    #[clap(long, global = true)]
    pub no_user_output: bool,
}

#[derive(Debug, Args)]
#[clap(next_help_heading = "DEBUG OUTPUT OPTIONS", setting = AppSettings::DeriveDisplayOrder)]
pub struct DebugOutputOpts {
    /// Disable logging
    #[clap(long, global = true)]
    pub no_log: bool,

    /// Specify where the program should output its logs
    #[clap(
        long,
        arg_enum,
        default_value_t,
        value_name = "LOG TARGET",
        global = true
    )]
    pub log_target: TracingTargetOption,

    /// Specify the severity of logs which should be
    #[clap(long, default_value_t, value_name = "LEVEL", global = true)]
    pub log_level: LogLevel,
}
