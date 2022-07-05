use crate::reporter::event::Message;
use crate::Event;
use owo_colors::OwoColorize;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Meta {
    instance: &'static str,
    version: &'static str,
    sha_short: &'static str,
    target_triple: &'static str,
    cargo_features: &'static str,
    rustc: &'static str,
}

const UNDEFINED: &str = "undefined";

impl Default for Meta {
    fn default() -> Self {
        Self {
            instance: option_env!("CARGO_PKG_NAME").unwrap_or("cargo-msrv"),
            version: option_env!("CARGO_PKG_VERSION").unwrap_or(UNDEFINED),
            sha_short: option_env!("VERGEN_GIT_SHA_SHORT").unwrap_or(UNDEFINED),
            target_triple: option_env!("VERGEN_CARGO_TARGET_TRIPLE").unwrap_or(UNDEFINED),
            cargo_features: option_env!("VERGEN_CARGO_FEATURES").unwrap_or(UNDEFINED),
            rustc: option_env!("VERGEN_RUSTC_SEMVER").unwrap_or(UNDEFINED),
        }
    }
}

impl Meta {
    pub fn summary(&self) -> String {
        format!(
            "  [{}] {} {} ({})",
            "Meta".bright_blue(),
            self.instance,
            self.version,
            self.sha_short,
        )
    }
}

impl From<Meta> for Event {
    fn from(it: Meta) -> Self {
        Message::Meta(it).into()
    }
}
