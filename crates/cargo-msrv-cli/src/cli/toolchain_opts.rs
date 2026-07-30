use clap::Args;

// Cli Options for commands which invoke Rust toolchains, such as the top level cargo msrv command
// (find) or cargo msrv verify
#[derive(Debug, Args)]
#[command(next_help_heading = "Toolchain options")]
pub struct ToolchainOpts {
    /// Check against a custom target (instead of the rustup default)
    // Unfortunately, Clap will not reject the
    #[arg(long, value_name = "TARGET", global = true)]
    pub target: Option<String>,

    /// Components be added to the toolchain
    ///
    /// Can be supplied multiple times to add multiple components.
    ///
    /// For example: --component rustc --component cargo
    #[arg(long, value_name = "COMPONENT", global = true)]
    pub component: Vec<String>,
}
