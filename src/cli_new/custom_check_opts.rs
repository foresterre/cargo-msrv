use clap::AppSettings;
use clap::Args;

#[derive(Debug, Args)]
#[clap(next_help_heading = "CUSTOM CHECK OPTIONS", setting = AppSettings::DeriveDisplayOrder)]
pub struct CustomCheckOpts {
    /// Supply a custom `check` command to be used by cargo msrv
    #[clap(last = true, required = false)]
    pub custom_check_command: Vec<String>,
}
