use crate::Event;
use crate::event::Message;

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
    pub fn new(
        instance: &'static str,
        version: &'static str,
        sha_short: Option<&'static str>,
        target_triple: Option<&'static str>,
        cargo_features: Option<&'static str>,
        rustc: Option<&'static str>,
    ) -> Self {
        Self {
            instance,
            version,
            sha_short,
            target_triple,
            cargo_features,
            rustc,
        }
    }

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

impl From<Meta> for Event {
    fn from(it: Meta) -> Self {
        Message::Meta(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestReporterWrapper;
    use crate::event::Message;
    use storyteller::EventReporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = Meta::new(
            "cargo-msrv",
            "1.2.3",
            Some("aaa1111"),
            Some("x86_64-unknown-linux-gnu"),
            Some("default"),
            Some("1.91.1"),
        );

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::Meta(event)),]
        );
    }
}
