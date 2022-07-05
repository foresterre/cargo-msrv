use crate::config::SearchMethod as Method;
use crate::reporter::event::Message;
use crate::Event;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FindMSRV {
    search_method: Method,
}

impl FindMSRV {
    pub(crate) fn new(method: Method) -> Self {
        Self {
            search_method: method,
        }
    }
}

impl From<FindMSRV> for Event {
    fn from(it: FindMSRV) -> Self {
        Message::FindMSRV(it).into()
    }
}
