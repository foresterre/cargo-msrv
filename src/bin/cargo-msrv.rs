use std::convert::TryFrom;
use std::ffi::OsString;
use std::io;
use std::sync::Arc;

use storyteller::{EventHandler, EventListener, FinishProcessing};

use cargo_msrv::cli::CargoCli;
use cargo_msrv::config::{Config, OutputFormat};
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
            Ok(exit_code) => exit_code,
            Err(err) => {
                eprintln!("error: {}", err);
                ExitCode::Failure
            }
        }
        .into(),
    );
}

fn _main<I: IntoIterator<Item = OsString>, F: FnOnce() -> I + Clone>(
    args: F,
) -> Result<ExitCode, InstanceError> {
    let matches = CargoCli::parse_args(args());
    let config = Config::try_from(&matches).map_err(InstanceError::CargoMsrv)?;

    init_and_run(&config)
}

fn init_and_run(config: &Config) -> Result<ExitCode, InstanceError> {
    let setup = ReporterSetup::default();
    let (reporter, listener) = setup.create();

    let handler = WrappingHandler::from(config.output_format());
    let finalizer = listener.run_handler(Arc::new(handler));
    let res = run_app(config, &reporter);

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
    Minimal(MinimalOutputHandler<io::Stdout, io::Stderr>),
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

    Ok(())
}

/// Wait for the user output processing to finish up it's queue of events by blocking.
fn wait_for_user_output(finalizer: impl FinishProcessing) -> Result<(), InstanceError> {
    finalizer
        .finish_processing()
        .map_err(|_| InstanceError::StorytellerFinishEventProcessing)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum InstanceError {
    // Only for compat. with `Config::try_from`, which is not as easily converted to this Error type
    // of the bin crate.
    // For that reason, we do not derive `#[from] CargoMSRVError`, so we don't silently miss calls
    // which may produce an `Err(CargoMSRVError)` where we do not want it.
    #[error("{0}")]
    CargoMsrv(CargoMSRVError),

    #[error("Failed to disconnect user output channel (storyteller)")]
    StorytellerDisconnect,

    #[error("Failed to send event to user output channel (storyteller)")]
    StorytellerSend,

    #[error("Failure while waiting for unprocessed events to be processed")]
    StorytellerFinishEventProcessing,
}
