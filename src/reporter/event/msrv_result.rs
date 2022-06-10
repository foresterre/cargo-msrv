use crate::config::{Config, SearchMethod};
use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::toolchain::OwnedToolchainSpec;
use crate::typed_bool::{False, True};
use crate::{semver, Event};
use std::path::PathBuf;

#[derive(serde::Serialize, Clone)]
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

            result: ResultDetails::Msrv {
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

            result: ResultDetails::None { success: False },
        }
    }

    pub fn msrv(&self) -> Option<&semver::Version> {
        if let Self {
            result: ResultDetails::Msrv { version, .. },
            ..
        } = self
        {
            Some(version)
        } else {
            None
        }
    }
}

impl IntoIdentifiableEvent for MsrvResult {
    fn identifier(&self) -> &'static str {
        "msrv_result"
    }
}

impl From<MsrvResult> for Event {
    fn from(it: MsrvResult) -> Self {
        Message::MsrvResult(it).into()
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
enum ResultDetails {
    Msrv {
        version: semver::Version,
        success: True,
    },
    None {
        success: False,
    },
}
