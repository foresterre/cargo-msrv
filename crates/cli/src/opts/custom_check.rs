use clap::Args;

#[derive(Debug, Args)]
#[command(next_help_heading = "Custom check options")]
pub struct CustomCheckOpts {
    /// Supply a custom `check` command to be used by cargo msrv
    #[arg(last = true, required = false)]
    pub custom_check_command: Vec<String>,
}
