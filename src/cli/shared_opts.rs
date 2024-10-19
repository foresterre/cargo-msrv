use crate::context::{OutputFormat, TracingTargetOption};
use crate::log_level::LogLevel;
use clap::{ArgGroup, Args, ValueHint};
use std::path::PathBuf;

// Cli Options shared between subcommands
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("paths").args(&["path", "manifest_path"])))]
pub struct SharedOpts {
    /// Path to project root directory
    ///
    /// This should be used over `--manifest-path` if not in a Cargo project.
    /// If you have a Cargo project, prefer `--manifest-path`.
    #[arg(long, value_name = "Crate Directory", global = true, value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,

    /// Path to cargo manifest file
    #[arg(long, value_name = "Cargo Manifest", global = true, value_hint = ValueHint::FilePath)]
    pub manifest_path: Option<PathBuf>,

    #[command(flatten)]
    pub workspace: clap_cargo::Workspace,

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
    output_format: OutputFormat,

    /// Disable user output
    #[arg(long, global = true, conflicts_with = "output_format")]
    no_user_output: bool,
}

impl UserOutputOpts {
    pub fn effective_output_format(&self) -> OutputFormat {
        if self.no_user_output {
            OutputFormat::None
        } else {
            self.output_format
        }
    }
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
