use clap::Args;

#[derive(Debug, Args)]
#[command(next_help_heading = "Custom check options")]
pub struct CargoCheckOpts {
    #[arg(long)]
    pub features: Option<Vec<String>>,

    #[arg(long, value_delimiter = ' ')]
    pub all_features: bool,

    #[arg(long)]
    pub no_default_features: bool,

    /// Supply a custom `check` command to be used by cargo msrv
    #[arg(last = true)]
    pub custom_check_command: Option<Vec<String>>,
}
