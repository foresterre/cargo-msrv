use std::convert::TryFrom;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use storyteller::{EventListener, Reporter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use cargo_msrv::cli::CargoCli;
use cargo_msrv::config::{self, Config, ModeIntent, TracingOptions, TracingTargetOption};
use cargo_msrv::errors::{CargoMSRVError, TResult};
use cargo_msrv::exit_code::ExitCode;
use cargo_msrv::run_app;
use cargo_msrv::storyteller::{
    DiscardOutputHandler, Event, HumanProgressHandler, JsonHandler, StorytellerSetup,
};

fn main() {
    std::process::exit(
        match _main(std::env::args_os) {
            Ok(_guard) => ExitCode::Success,
            Err(err) => {
                tracing::error!("{}", err);
                ExitCode::Failure
            }
        }
        .into(),
    );
}

fn _main<I: IntoIterator<Item = OsString>, F: FnOnce() -> I + Clone>(
    args: F,
) -> TResult<Option<TracingGuard>> {
    let matches = CargoCli::parse_args(args());

    let config = Config::try_from(&matches)?;

    // NB: We must collect the guard of the non-blocking tracing appender, since it will only live as
    // long as the lifetime of the worker guard. If we don't do this, the guard would be dropped after
    // the scope of `if !config.no_tracing() { ... }` ended, and as a result, anything logged in
    // `init_and_run` would not be logged.
    let mut guard = Option::None;

    if let Some(options) = config.tracing() {
        let tracing_config = TracingConfig::try_from_options(options)?;
        guard = Some(init_tracing(&tracing_config)?);
    }

    init_and_run(&config)?;

    Ok(guard)
}

fn init_and_run(config: &Config) -> TResult<()> {
    tracing::info!(
        cargo_msrv_version = env!("CARGO_PKG_VERSION"),
        "initializing"
    );

    // todo!
    let storyteller = StorytellerSetup::new();
    let (reporter, listener) = storyteller.create_channels::<Event>();

    match config.output_format() {
        config::OutputFormat::Human => {
            let custom_cmd = config.check_command_string();
            // todo!
            // let reporter = reporter::ui::HumanPrinter::new(1, config.target(), &custom_cmd);
            let handler = HumanProgressHandler::new();
            listener.run_handler(handler);

            run_app(config, &reporter)
        }
        config::OutputFormat::Json => {
            // todo!
            // let custom_cmd = if let ModeIntent::List = config.action_intent() {
            //     None
            // } else {
            //     Some(config.check_command_string())
            // };
            //
            // let reporter =
            //     reporter::json::JsonPrinter::new(1, config.target(), custom_cmd.as_deref());
            let handler = JsonHandler::stderr();
            listener.run_handler(handler);

            run_app(config, &reporter)
        }
        config::OutputFormat::None => {
            // To disable regular output. Useful when outputting logs to stdout, as the
            //   regular output and the log output may otherwise interfere with each other.
            let handler = DiscardOutputHandler;
            listener.run_handler(handler);

            run_app(config, &reporter)
        }
    }?;

    tracing::info!("finished");

    // todo! handle error
    let _ = reporter.disconnect();

    Ok(())
}

fn init_tracing(tracing_config: &TracingConfig) -> TResult<TracingGuard> {
    let level = tracing_config.level;

    match &tracing_config.target {
        // Log (non-blocking) to disk
        TracingTarget::ToDisk(path) => {
            let guard = init_tracing_to_file(path, level);

            let folder = format!("{}", path.display());
            tracing::debug!(log_folder = folder.as_str());

            guard
        }
        // Log to stdout
        TracingTarget::Stdout => init_tracing_to_stdout(level),
    }
}

fn init_tracing_to_file(
    log_folder: impl AsRef<Path>,
    level: tracing::Level,
) -> TResult<TracingGuard> {
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_folder, "cargo-msrv-log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(level)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| CargoMSRVError::UnableToInitTracing)?;

    Ok(TracingGuard::NonBlockingGuard(guard))
}

fn init_tracing_to_stdout(level: tracing::Level) -> TResult<TracingGuard> {
    let subscriber = tracing_subscriber::fmt().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| CargoMSRVError::UnableToInitTracing)?;

    Ok(TracingGuard::None)
}

struct TracingConfig {
    level: tracing::Level,
    target: TracingTarget,
}

impl TracingConfig {
    fn try_from_options(config: &TracingOptions) -> TResult<Self> {
        let target = TracingTarget::try_from_option(config.target())?;

        Ok(Self {
            level: (*config.level()).into(),
            target,
        })
    }
}

enum TracingTarget {
    ToDisk(PathBuf),
    Stdout,
}

impl TracingTarget {
    fn try_from_option(option: &TracingTargetOption) -> TResult<Self> {
        match option {
            TracingTargetOption::File => {
                let folder = log_folder()?;
                Ok(Self::ToDisk(folder))
            }
            TracingTargetOption::Stdout => Ok(Self::Stdout),
        }
    }
}

enum TracingGuard {
    NonBlockingGuard(tracing_appender::non_blocking::WorkerGuard),
    None,
}

fn log_folder() -> TResult<PathBuf> {
    dirs::data_local_dir()
        .map(|path| path.join("cargo-msrv"))
        .ok_or(CargoMSRVError::UnableToAccessLogFolder)
}
