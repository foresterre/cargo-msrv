use clap::AppSettings;
use clap::Args;

// Cli Options for commands which invoke Rust toolchains, such as the top level cargo msrv command
// (find) or cargo msrv verify
#[derive(Debug, Args)]
#[clap(next_help_heading = "TOOLCHAIN OPTIONS", setting = AppSettings::DeriveDisplayOrder)]
pub struct ToolchainOpts {
    /// Check against a custom target (instead of the rustup default)
    #[clap(long, value_name = "TARGET")]
    pub target: Option<String>,
}
