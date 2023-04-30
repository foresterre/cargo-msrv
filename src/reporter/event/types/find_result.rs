use crate::context::SearchMethod;
use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::Message;
use crate::typed_bool::{False, True};
use crate::{semver, Event};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FindResult {
    #[serde(skip)]
    pub target: String,
    #[serde(skip)]
    pub minimum_version: BareVersion,
    #[serde(skip)]
    pub maximum_version: BareVersion,
    #[serde(skip)]
    pub search_method: SearchMethod,

    result: ResultDetails,
}

impl FindResult {
    pub fn new_msrv(
        version: semver::Version,
        target: impl Into<String>,
        min: BareVersion,
        max: BareVersion,
        search_method: SearchMethod,
    ) -> Self {
        Self {
            target: target.into(),
            minimum_version: min,
            maximum_version: max,

            search_method,

            result: ResultDetails::Determined {
                version,
                success: True,
            },
        }
    }

    pub fn none(
        target: impl Into<String>,
        min: BareVersion,
        max: BareVersion,
        search_method: SearchMethod,
    ) -> Self {
        Self {
            target: target.into(),
            minimum_version: min,
            maximum_version: max,

            search_method,

            result: ResultDetails::Undetermined { success: False },
        }
    }

    pub fn msrv(&self) -> Option<&semver::Version> {
        if let Self {
            result: ResultDetails::Determined { version, .. },
            ..
        } = self
        {
            Some(version)
        } else {
            None
        }
    }
}

impl From<FindResult> for SubcommandResult {
    fn from(it: FindResult) -> Self {
        SubcommandResult::Find(it)
    }
}

impl From<FindResult> for Event {
    fn from(it: FindResult) -> Self {
        Message::SubcommandResult(it.into()).into()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum ResultDetails {
    Determined {
        version: semver::Version,
        success: True,
    },
    Undetermined {
        success: False,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use storyteller::EventReporter;

    #[test]
    fn reported_msrv_determined_event() {
        let reporter = TestReporterWrapper::default();
        let version = semver::Version::new(1, 3, 0);
        let min = BareVersion::TwoComponents(1, 0);
        let max = BareVersion::ThreeComponents(1, 4, 0);

        let event = FindResult::new_msrv(version, "x", min, max, SearchMethod::Linear);
        reporter.get().report_event(event.clone()).unwrap();

        let events = reporter.wait_for_events();
        assert_eq!(
            &events,
            &[Event::unscoped(Message::SubcommandResult(
                SubcommandResult::Find(event)
            ))]
        );

        let inner = &events[0].message;
        assert!(
            matches!(inner, Message::SubcommandResult(SubcommandResult::Find(res)) if res.msrv() == Some(&semver::Version::new(1, 3, 0)))
        );
    }

    #[test]
    fn reported_msrv_undetermined_event() {
        let reporter = TestReporterWrapper::default();
        let min = BareVersion::TwoComponents(1, 0);
        let max = BareVersion::ThreeComponents(1, 4, 0);

        let event = FindResult::none("x", min, max, SearchMethod::Linear);

        reporter.get().report_event(event.clone()).unwrap();

        let events = reporter.wait_for_events();
        assert_eq!(
            &events,
            &[Event::unscoped(Message::SubcommandResult(
                SubcommandResult::Find(event)
            ))]
        );

        let inner = &events[0].message;
        assert!(
            matches!(inner, Message::SubcommandResult(SubcommandResult::Find(res)) if res.msrv().is_none())
        );
    }
}
