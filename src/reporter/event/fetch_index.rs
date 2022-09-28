use crate::reporter::event::Message;
use crate::{Event, ReleaseSource};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FetchIndex {
    #[serde(rename = "source")]
    from_source: ReleaseSource,
}

impl FetchIndex {
    pub fn new(source: ReleaseSource) -> Self {
        Self {
            from_source: source,
        }
    }
}

impl From<FetchIndex> for Event {
    fn from(it: FetchIndex) -> Self {
        Message::FetchIndex(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use storyteller::EventReporter;

    #[test]
    fn reported_rust_changelog_source() {
        let reporter = TestReporterWrapper::default();
        let event = FetchIndex::new(ReleaseSource::RustChangelog);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::FetchIndex(event)),]
        );
    }

    #[cfg(feature = "rust-releases-dist-source")]
    #[test]
    fn reported_rust_dist_source() {
        let reporter = TestReporterWrapper::default();
        let event = FetchIndex::new(ReleaseSource::RustDist);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::FetchIndex(event)),]
        );
    }
}
