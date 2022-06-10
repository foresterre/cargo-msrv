use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::{Event, ReleaseSource};
use owo_colors::OwoColorize;

#[derive(serde::Serialize, Clone)]
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

impl IntoIdentifiableEvent for FetchIndex {
    fn identifier(&self) -> &'static str {
        "fetch_index"
    }
}

impl From<FetchIndex> for Event {
    fn from(it: FetchIndex) -> Self {
        Message::FetchIndex(it).into()
    }
}
