use clap::{ArgGroup, Args, ValueHint};
use std::path::PathBuf;

// Cli Options shared between subcommands
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("paths").args(&["path", "manifest_path"])))]
pub struct SharedOpts {
    /// Path to cargo project directory
    #[arg(long, value_name = "Crate Directory", global = true, value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,

    /// Path to cargo manifest file
    #[arg(long, value_name = "Cargo Manifest", global = true, value_hint = ValueHint::FilePath)]
    pub manifest_path: Option<PathBuf>,

    #[command(flatten)]
    pub user_output_opts: UserOutputOpts,

    #[command(flatten)]
    pub debug_output_opts: DebugOutputOpts,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "User output options")]
pub struct UserOutputOpts {
    /// Set the format of user output
    #[arg(
        long,
        value_enum,
        default_value_t,
        value_name = "FORMAT",
        global = true
    )]
    pub output_format: OutputFormat,

    /// Disable user output
    #[arg(long, global = true)]
    pub no_user_output: bool,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Debug output options")]
pub struct DebugOutputOpts {
    /// Disable logging
    #[arg(long, global = true)]
    pub no_log: bool,

    /// Specify where the program should output its logs
    #[arg(
        long,
        value_enum,
        default_value_t,
        value_name = "LOG TARGET",
        global = true
    )]
    pub log_target: TracingTargetOption,

    /// Specify the severity of logs which should be
    #[arg(long, value_enum, default_value_t, value_name = "LEVEL", global = true)]
    pub log_level: LogLevel,
}
