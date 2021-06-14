use cargo_msrv::config::Config;
use cargo_msrv::errors::TResult;
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

    run_app(&config)
}
