use clap::Args;

#[derive(Debug, Args)]
#[command(next_help_heading = "Custom check options")]
pub struct CheckCommandOpts {
    /// Forwards the provided features to cargo, when running cargo-msrv with the default compatibility
    /// check command.
    ///
    /// If a custom compatibility check command is used, this option is ignored.
    #[arg(long)]
    pub features: Option<Vec<String>>,

    /// Forwards the --all-features flag to cargo, when running cargo-msrv with the default compatibility
    /// check command.
    ///
    /// If a custom compatibility check command is used, this option is ignored.
    #[arg(long, value_delimiter = ' ')]
    pub all_features: bool,

    /// Forwards the --no-default-features flag to cargo, when running cargo-msrv with the default compatibility
    /// check command.
    ///
    /// If a custom compatibility check command is used, this option is ignored.
    #[arg(long)]
    pub no_default_features: bool,

    /// Supply a custom `check` command to be used by cargo msrv
    #[arg(last = true)]
    pub custom_check_command: Option<Vec<String>>,
}
