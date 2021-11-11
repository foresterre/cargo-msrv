use std::convert::TryFrom;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;

use cargo_msrv::config::{self, Config, ModeIntent};
use cargo_msrv::errors::{CargoMSRVError, TResult};
use cargo_msrv::reporter;
use cargo_msrv::{cli, run_app};

fn main() {
    if let Err(err) = _main(args) {
        eprintln!("{}", err);
    }
}

// When we call cargo-msrv with cargo, cargo will supply the msrv subcommand, in addition
// to the binary name itself. As a result, when you call cargo-msrv without cargo, for example
// `cargo-msrv` (without cargo) instead of `cargo msrv` (with cargo), the process will receive
// too many arguments, and you will have to specify the subcommand again like so: `cargo-msrv msrv`.
// This function removes the subcommand when it's present in addition to the program name.
fn args() -> impl IntoIterator<Item = String> {
    fn cargo_subcommand_name(cargo_subcommand: &str) -> String {
        if cfg!(target_os = "windows") {
            format!("cargo-{}.exe", cargo_subcommand)
        } else {
            format!("cargo-{}", cargo_subcommand)
        }
    }
    let mut args = std::env::args().collect::<Vec<_>>();

    if args.len() >= 2 {
        let program = args[0].as_str();
        let subcommand = args[1].as_str();

        // when `cargo-msrv` and `msrv` are present
        if program.ends_with(&cargo_subcommand_name(subcommand)) {
            // remove `msrv`
            args.remove(1);
        }
    }

    args
}

fn _main<I: IntoIterator<Item = String>, F: FnOnce() -> I>(
    args: F,
) -> TResult<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let matches = cli::cli().get_matches_from(args());
    let config = Config::try_from(&matches)?;

    // NB: We must collect the guard of the non-blocking tracing appender, since it will only live as
    // long as the lifetime of the worker guard. If we don't do this, the guard would be dropped after
    // the scope of `if !config.no_tracing() { ... }` ended, and as a result, anything logged in
    // `init_and_run` would not be logged.
    let mut guard = Option::None;

    if !config.no_tracing() {
        guard = Some(init_tracing()?);
    }

    init_and_run(&config)?;

    Ok(guard)
}

fn init_and_run(config: &Config) -> TResult<()> {
    tracing::info!("Running app");

    match config.output_format() {
        config::OutputFormat::Human => {
            let custom_cmd = config.check_command_string();
            let reporter = reporter::ui::HumanPrinter::new(1, config.target(), &custom_cmd);
            run_app(config, &reporter)
        }
        config::OutputFormat::Json => {
            let custom_cmd = if let ModeIntent::List = config.action_intent() {
                None
            } else {
                Some(config.check_command_string())
            };

            let reporter =
                reporter::json::JsonPrinter::new(1, config.target(), custom_cmd.as_deref());
            run_app(config, &reporter)
        }
        config::OutputFormat::None => {
            // for testing without any output
            let reporter = reporter::__private::NoOutput;

            run_app(config, &reporter)
        }
        config::OutputFormat::TestSuccesses => {
            // for collecting success results during testing
            let reporter = reporter::__private::SuccessOutput::default();
            run_app(config, &reporter)
        }
    }?;

    tracing::info!("Finished app");

    Ok(())
}

fn init_tracing() -> TResult<tracing_appender::non_blocking::WorkerGuard> {
    let log_folder = dirs::data_local_dir()
        .map(|path| path.join("cargo-msrv"))
        .ok_or(CargoMSRVError::UnableToAccessLogFolder)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_folder, "cargo-msrv-log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(LevelFilter::INFO)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| CargoMSRVError::UnableToInitTracing)?;

    Ok(guard)
}
