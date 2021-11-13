use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use tracing_appender::rolling::{RollingFileAppender, Rotation};

use cargo_msrv::config::{self, Config, ModeIntent, TracingOptions, TracingTargetOption};
use cargo_msrv::errors::{CargoMSRVError, TResult};
use cargo_msrv::reporter;
use cargo_msrv::{cli, run_app};

enum ExitCode {
    Success,
    Failure,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        match code {
            ExitCode::Success => 0,
            ExitCode::Failure => 1,
        }
    }
}

fn main() {
    std::process::exit(
        match _main(args) {
            Ok(_guard) => ExitCode::Success,
            Err(err) => {
                eprintln!("{}", err);
                ExitCode::Failure
            }
        }
        .into(),
    );
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
) -> TResult<Option<TracingGuard>> {
    let matches = cli::cli().get_matches_from(args());
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
            // To disable regular output. Useful when outputting logs to stdout, as the
            //   regular output and the log output may otherwise interfere with each other.
            let reporter = reporter::no_output::NoOutput;

            run_app(config, &reporter)
        }
        config::OutputFormat::TestSuccesses => {
            // for collecting success results during testing
            let reporter = reporter::__private::SuccessOutput::default();
            run_app(config, &reporter)
        }
    }?;

    tracing::info!("finished");

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

        Ok(TracingConfig {
            level: *config.level(),
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
                Ok(TracingTarget::ToDisk(folder))
            }
            TracingTargetOption::Stdout => Ok(TracingTarget::Stdout),
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
