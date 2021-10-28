use cargo_msrv::config::{self, Config};
use cargo_msrv::errors::{CargoMSRVError, TResult};
use cargo_msrv::reporter;
use cargo_msrv::{cli, run_app};
use std::convert::TryFrom;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;

fn main() {
    if let Err(err) = _main() {
        eprintln!("{}", err);
    }
}

fn _main() -> TResult<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let matches = cli::cli().get_matches();
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

    let _ = match config.output_format() {
        config::OutputFormat::Human => {
            let custom_cmd = config.check_command_string();
            let reporter = reporter::ui::HumanPrinter::new(1, config.target(), &custom_cmd);
            run_app(config, &reporter)
        }
        config::OutputFormat::Json => {
            let custom_cmd = config.check_command_string();
            let reporter = reporter::json::JsonPrinter::new(1, config.target(), &custom_cmd);
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
