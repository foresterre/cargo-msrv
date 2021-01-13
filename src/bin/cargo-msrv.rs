extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use cargo_msrv::errors::CargoMSRVError;
use cargo_msrv::run_cargo_msrv;
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;
use std::str::FromStr;

fn main() {
    let level_filter = std::env::var("RUST_LOG")
        .map_err(CargoMSRVError::Env)
        .and_then(|var| {
            eprintln!("Unable to set output log level, using the default (info)");

            LevelFilter::from_str(&var).map_err(CargoMSRVError::Log)
        })
        .unwrap_or(LevelFilter::Info);

    if let Err(e) = formatted_timed_builder()
        .filter_level(level_filter)
        .try_init()
    {
        eprintln!("Unable to enable log output: {}", e);
    }

    if let Err(err) = run_cargo_msrv() {
        error!("{}", err);
    }
}
