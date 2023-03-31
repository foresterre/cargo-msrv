use crate::reporter::event::Message;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Meta {
    instance: &'static str,
    version: &'static str,
    sha_short: Option<&'static str>,
    target_triple: Option<&'static str>,
    cargo_features: Option<&'static str>,
    rustc: Option<&'static str>,
}

impl Meta {
    pub fn instance(&self) -> &'static str {
        self.instance
    }

    pub fn version(&self) -> &'static str {
        self.version
    }

    pub fn sha_short(&self) -> Option<&'static str> {
        self.sha_short
    }
    pub fn target_triple(&self) -> Option<&'static str> {
        self.target_triple
    }

    pub fn cargo_features(&self) -> Option<&'static str> {
        self.cargo_features
    }

    pub fn rustc(&self) -> Option<&'static str> {
        self.rustc
    }
}

const UNKNOWN_VERSION: &str = "?";

impl Default for Meta {
    fn default() -> Self {
        Self {
            instance: option_env!("CARGO_PKG_NAME").unwrap_or("cargo-msrv"),
            version: option_env!("CARGO_PKG_VERSION").unwrap_or(UNKNOWN_VERSION),
            sha_short: option_env!("VERGEN_GIT_SHA_SHORT"),
            target_triple: option_env!("VERGEN_CARGO_TARGET_TRIPLE"),
            cargo_features: option_env!("VERGEN_CARGO_FEATURES"),
            rustc: option_env!("VERGEN_RUSTC_SEMVER"),
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
    use storyteller::EventReporter;

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
