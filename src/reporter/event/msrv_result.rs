use crate::config::{Config, SearchMethod};
use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::Message;
use crate::typed_bool::{False, True};
use crate::{semver, Event};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MsrvResult {
    #[serde(skip)]
    pub target: String,
    #[serde(skip)]
    pub minimum_version: BareVersion,
    #[serde(skip)]
    pub maximum_version: BareVersion,
    #[serde(skip)]
    pub search_method: SearchMethod,

    #[serde(flatten)]
    result: ResultDetails,
}

impl MsrvResult {
    pub fn new_msrv(
        version: semver::Version,
        config: &Config,
        min: BareVersion,
        max: BareVersion,
    ) -> Self {
        Self {
            target: config.target().to_string(),
            minimum_version: config
                .minimum_version()
                .map(Clone::clone)
                .unwrap_or_else(|| min),
            maximum_version: config
                .maximum_version()
                .map(Clone::clone)
                .unwrap_or_else(|| max),

            search_method: config.search_method(),

            result: ResultDetails::Determined {
                version,
                success: True,
            },
        }
    }

    pub fn none(config: &Config, min: BareVersion, max: BareVersion) -> Self {
        Self {
            target: config.target().to_string(),
            minimum_version: config
                .minimum_version()
                .map(Clone::clone)
                .unwrap_or_else(|| min),
            maximum_version: config
                .maximum_version()
                .map(Clone::clone)
                .unwrap_or_else(|| max),

            search_method: config.search_method(),

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

impl From<MsrvResult> for Event {
    fn from(it: MsrvResult) -> Self {
        Message::MsrvResult(it).into()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum ResultDetails {
    Determined {
        version: semver::Version,
        success: True,
    },
    Undetermined {
        success: False,
    },
}
