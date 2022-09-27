use crate::reporter::event::Message;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Meta {
    instance: &'static str,
    version: &'static str,
    sha_short: &'static str,
    target_triple: &'static str,
    cargo_features: &'static str,
    rustc: &'static str,
}

impl Meta {
    pub fn instance(&self) -> &'static str {
        self.instance
    }
    pub fn version(&self) -> &'static str {
        self.version
    }
    pub fn sha_short(&self) -> &'static str {
        self.sha_short
    }
    pub fn target_triple(&self) -> &'static str {
        self.target_triple
    }
    pub fn cargo_features(&self) -> &'static str {
        self.cargo_features
    }
    pub fn rustc(&self) -> &'static str {
        self.rustc
    }
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

impl From<Meta> for Event {
    fn from(it: Meta) -> Self {
        Message::Meta(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use storyteller::Reporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = Meta::default();

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::Meta(event)),]
        );
    }
}
