extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use cargo_msrv::{init_logger, run_cargo_msrv};

fn main() {
    init_logger();

    // ensure a user will see output produced by the logger
    if !cfg!(debug_assertions) && option_env!("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    if let Err(err) = run_cargo_msrv() {
        error!("{}", err);
    }
}
