use camino::Utf8PathBuf;
use cargo_msrv_rust_releases::ExcludedRelease;
use cargo_msrv_types::BareVersion;
use owo_colors::OwoColorize;

pub use cargo_msrv_context::context::error::{IoError, IoErrorSource};

pub type TResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    LockfileHandler(#[from] LockfileHandlerError),

    #[error(transparent)]
    NoToolchainsToTry(#[from] NoToolchainsToTryError),

    #[error(transparent)]
    Rustup(#[from] RustupError),

    #[error("Unable to print event output")]
    Storyteller,

    #[error("Unable to run the check command: '{}' at '{}'", &command, &cwd)]
    UnableToRunCheck { command: String, cwd: Utf8PathBuf },
}

impl<T> From<storyteller::EventReporterError<T>> for Error {
    fn from(_: storyteller::EventReporterError<T>) -> Self {
        Self::Storyteller
    }
}

#[derive(Debug, thiserror::Error)]
#[error("No Rust releases to check: the filtered search space is empty.{}{}",
    inner.as_ref().map(|clues| format!("{}", clues)).unwrap_or_default(),
    unavailable.as_ref().map(|info| format!("{}", info)).unwrap_or_default(),
)]
pub struct NoToolchainsToTryError {
    inner: Option<SelectedMinMaxVersion>,
    unavailable: Option<Box<UnavailableToolchainDetails>>,
}

impl NoToolchainsToTryError {
    pub fn new_empty() -> Self {
        Self {
            inner: None,
            unavailable: None,
        }
    }

    pub fn with_details(user_min: Option<BareVersion>, user_max: Option<BareVersion>) -> Self {
        Self {
            inner: Some(SelectedMinMaxVersion {
                min: user_min,
                max: user_max,
            }),
            unavailable: None,
        }
    }

    pub fn with_unavailable_toolchains(
        mut self,
        target: &str,
        components: &[&str],
        excluded: &[ExcludedRelease],
    ) -> Self {
        self.unavailable = Some(Box::new(UnavailableToolchainDetails {
            target: target.to_string(),
            components: components.iter().map(|c| c.to_string()).collect(),
            excluded: excluded.to_vec(),
        }));

        self
    }

    pub fn has_clues(&self) -> bool {
        self.inner.is_some() || self.unavailable.is_some()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(" Search space limited by user to min Rust '{}', and max Rust '{}'",
    min.as_ref().map(|s| format!("{}", s)).unwrap_or_else(|| "<not overridden>".to_string()),
    max.as_ref().map(|s| format!("{}", s)).unwrap_or_else(|| "<not overridden>".to_string()),
)]
pub struct SelectedMinMaxVersion {
    min: Option<BareVersion>,
    max: Option<BareVersion>,
}

const UNAVAILABLE_EXAMPLES: usize = 3;

#[derive(Debug, thiserror::Error)]
#[error(" Excluded {} release(s) which do not provide the requested toolchain (target '{}'{}): {}",
    excluded.len(),
    target,
    if components.is_empty() { String::new() } else { format!(", component(s) '{}'", components.join(", ")) },
    format_excluded(excluded),
)]
pub struct UnavailableToolchainDetails {
    target: String,
    components: Vec<String>,
    excluded: Vec<ExcludedRelease>,
}

fn format_excluded(excluded: &[ExcludedRelease]) -> String {
    let examples = excluded
        .iter()
        .take(UNAVAILABLE_EXAMPLES)
        .map(|release| format!("Rust {} ({})", release.version(), release.reason()))
        .collect::<Vec<_>>()
        .join(", ");

    match excluded.len().saturating_sub(UNAVAILABLE_EXAMPLES) {
        0 => examples,
        remainder => format!("{}, and {} more", examples, remainder),
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum RustupError {
    Install(#[from] RustupInstallError),
    AddComponent(#[from] RustupAddComponentError),
    AddTarget(#[from] RustupAddTargetError),
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to install toolchain '{}', rustup reported:\n    {}",
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupInstallError {
    pub toolchain_spec: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to add components '{}' to toolchain '{}', rustup reported:\n    {}",
    components,
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupAddComponentError {
    pub components: String,
    pub toolchain_spec: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to add target '{}' to toolchain '{}', rustup reported:\n    {}",
    targets,
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupAddTargetError {
    pub targets: String,
    pub toolchain_spec: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Unable to set cleanup handler for lockfile")]
pub struct LockfileHandlerError;
