use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::Event;
use owo_colors::OwoColorize;

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Meta {
    instance: &'static str,
    version: &'static str,
    sha_short: &'static str,
    target_triple: &'static str,
    cargo_features: &'static str,
    rustc: &'static str,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            instance: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            sha_short: env!("VERGEN_GIT_SHA_SHORT"),
            target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
            cargo_features: env!("VERGEN_CARGO_FEATURES"),
            rustc: env!("VERGEN_RUSTC_SEMVER"),
        }
    }
}

impl Meta {
    pub fn summary(&self) -> String {
        format!(
            "{:>12} {} {} ({})",
            "Meta".bright_green(),
            self.instance,
            self.version,
            self.sha_short,
        )
    }
}

impl IntoIdentifiableEvent for Meta {
    fn identifier(&self) -> &'static str {
        "meta"
    }
}

impl From<Meta> for Event {
    fn from(it: Meta) -> Self {
        Message::Meta(it).into_event()
    }
}
