use std::convert::TryFrom;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use storyteller::{EventHandler, EventListener, FinishProcessing};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use cargo_msrv::cli::CargoCli;
use cargo_msrv::config::{Config, OutputFormat, TracingOptions, TracingTargetOption};
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::exit_code::ExitCode;
use cargo_msrv::reporter::{
    DiscardOutputHandler, HumanProgressHandler, JsonHandler, MinimalOutputHandler, ReporterSetup,
};
use cargo_msrv::reporter::{Event, Reporter, TerminateWithFailure};
use cargo_msrv::run_app;

fn main() {
    std::process::exit(
        match _main(std::env::args_os) {
            Ok((_guard, exit_code)) => exit_code,
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
) -> Result<(Option<TracingGuard>, ExitCode), InstanceError> {
    let matches = CargoCli::parse_args(args());
    let config = Config::try_from(&matches).map_err(InstanceError::CargoMsrv)?;

    // NB: We must collect the guard of the non-blocking tracing appender, since it will only live as
    // long as the lifetime of the worker guard. If we don't do this, the guard would be dropped after
    // the scope of `if !config.no_tracing() { ... }` ended, and as a result, anything logged in
    // `init_and_run` would not be logged.
    let mut guard = None;

    if let Some(options) = config.tracing() {
        let tracing_config = TracingConfig::try_from_options(options)?;
        guard = Some(init_tracing(&tracing_config)?);
    }

    init_and_run(&config).map(|exit_code| (guard, exit_code))
}

fn init_and_run(config: &Config) -> Result<ExitCode, InstanceError> {
    tracing::info!(
        cargo_msrv_version = env!("CARGO_PKG_VERSION"),
        "initializing"
    );

    let setup = ReporterSetup::default();
    let (reporter, listener) = setup.create();

    tracing::info!("storyteller channel created");

    let handler = WrappingHandler::from(config.output_format());
    let finalizer = listener.run_handler(Arc::new(handler));
    tracing::info!("storyteller started handler");
    tracing::info!("start run_app");

    let res = run_app(config, &reporter);

    tracing::info!("finished run_app");

    let exit_code = get_exit_code(res, &reporter)?;
    disconnect_reporter(reporter)?;
    wait_for_user_output(finalizer)?;

    Ok(exit_code)
}

/// Get the exit code from the result of the program's main work unit.
fn get_exit_code(
    result: Result<(), CargoMSRVError>,
    reporter: &impl Reporter,
) -> Result<ExitCode, InstanceError> {
    Ok(match result {
        Ok(_) => ExitCode::Success,
        Err(err) => {
            reporter
                .report_event(TerminateWithFailure::new(err))
                .map_err(|_| InstanceError::StorytellerSend)?;

            ExitCode::Failure
        }
    })
}

/// Enumerates the in our program available output handlers, and implements EventHandler which
/// directly delegates the implementation to the wrapped handlers.
enum WrappingHandler {
    HumanProgress(HumanProgressHandler),
    Json(JsonHandler<io::Stderr>),
    Minimal(MinimalOutputHandler<io::Stderr>),
    DiscardOutput(DiscardOutputHandler),
}

impl EventHandler for WrappingHandler {
    type Event = Event;

    fn handle(&self, event: Self::Event) {
        match self {
            WrappingHandler::HumanProgress(inner) => inner.handle(event),
            WrappingHandler::Json(inner) => inner.handle(event),
            WrappingHandler::Minimal(inner) => inner.handle(event),
            WrappingHandler::DiscardOutput(inner) => inner.handle(event),
        }
    }

    fn finish(&self) {
        match self {
            WrappingHandler::HumanProgress(inner) => inner.finish(),
            WrappingHandler::Json(inner) => inner.finish(),
            WrappingHandler::Minimal(inner) => inner.finish(),
            WrappingHandler::DiscardOutput(inner) => inner.finish(),
        }
    }
}

impl From<OutputFormat> for WrappingHandler {
    fn from(output_format: OutputFormat) -> Self {
        match output_format {
            OutputFormat::Human => Self::HumanProgress(HumanProgressHandler::default()),
            OutputFormat::Json => Self::Json(JsonHandler::stderr()),
            OutputFormat::Minimal => Self::Minimal(MinimalOutputHandler::stderr()),
            OutputFormat::None => {
                // To disable regular output. Useful when outputting logs to stdout, as the
                //   regular output and the log output may otherwise interfere with each other.
                Self::DiscardOutput(DiscardOutputHandler)
            }
        }
    }
}

/// Disconnect the reporter, signalling that the program is finished, and we can now finish
/// up processing the last user output events.
fn disconnect_reporter(reporter: impl Reporter) -> Result<(), InstanceError> {
    reporter
        .disconnect()
        .map_err(|_| InstanceError::StorytellerDisconnect)?;

    tracing::info!("disconnected reporter");

    Ok(())
}

/// Wait for the user output processing to finish up it's queue of events by blocking.
fn wait_for_user_output(finalizer: impl FinishProcessing) -> Result<(), InstanceError> {
    finalizer
        .finish_processing()
        .map_err(|_| InstanceError::StorytellerFinishEventProcessing)?;

    tracing::info!("finished processing unprocessed events");

    Ok(())
}

fn init_tracing(tracing_config: &TracingConfig) -> Result<TracingGuard, InstanceError> {
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
) -> Result<TracingGuard, InstanceError> {
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_folder, "cargo-msrv-log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(level)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| InstanceError::UnableToInitTracing)?;

    Ok(TracingGuard::NonBlockingGuard(guard))
}

fn init_tracing_to_stdout(level: tracing::Level) -> Result<TracingGuard, InstanceError> {
    let subscriber = tracing_subscriber::fmt().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| InstanceError::UnableToInitTracing)?;

    Ok(TracingGuard::None)
}

struct TracingConfig {
    level: tracing::Level,
    target: TracingTarget,
}

impl TracingConfig {
    fn try_from_options(config: &TracingOptions) -> Result<Self, InstanceError> {
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
    fn try_from_option(option: &TracingTargetOption) -> Result<Self, InstanceError> {
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

fn log_folder() -> Result<PathBuf, InstanceError> {
    dirs::data_local_dir()
        .map(|path| path.join("cargo-msrv"))
        .ok_or(InstanceError::UnableToAccessLogFolder)
}

#[derive(Debug, thiserror::Error)]
enum InstanceError {
    // Only for compat. with `Config::try_from`, which is not as easily converted to this Error type
    // of the bin crate.
    // For that reason, we do not derive `#[from] CargoMSRVError`, so we don't silently miss calls
    // which may produce an `Err(CargoMSRVError)` where we do not want it.
    #[error("{0}")]
    CargoMsrv(CargoMSRVError),

    #[error("Unable to init logger, run with --no-log to try again without logging.")]
    UnableToInitTracing,

    #[error("Unable to access log folder, run with --no-log to try again without logging.")]
    UnableToAccessLogFolder,

    #[error("Failed to disconnect user output channel (storyteller)")]
    StorytellerDisconnect,

    #[error("Failed to send event to user output channel (storyteller)")]
    StorytellerSend,

    #[error("Failure while waiting for unprocessed events to be processed")]
    StorytellerFinishEventProcessing,
}
