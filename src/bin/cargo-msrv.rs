use cargo_msrv::config::Config;
use cargo_msrv::errors::TResult;
use cargo_msrv::reporter::ReporterBuilder;
use cargo_msrv::{cli, run_app};
use std::convert::TryFrom;

fn main() {
    if let Err(err) = init_and_run() {
        eprintln!("{}", err);
    }
}

fn init_and_run() -> TResult<()> {
    let matches = cli::cli().get_matches();
    let config = Config::try_from(&matches)?;

    let target = config.target().as_str();
    let cmd = config.check_command_string();
    let reporter = ReporterBuilder::new(target, cmd.as_str())
        .output_format(config.output_format())
        .build();

    run_app(&config, &reporter)
}
