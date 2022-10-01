use clap::Args;

// Cli Options for commands which invoke Rust toolchains, such as the top level cargo msrv command
// (find) or cargo msrv verify
#[derive(Debug, Args)]
#[command(next_help_heading = "Toolchain options")]
pub struct ToolchainOpts {
    /// Check against a custom target (instead of the rustup default)
    #[arg(long, value_name = "TARGET")]
    pub target: Option<String>,
}
