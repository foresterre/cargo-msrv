use crate::config::SearchMethod as Method;
use crate::reporter::event::Message;
use crate::Event;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Search {
    search_method: Method,
}

impl Search {
    pub(crate) fn new(method: Method) -> Self {
        Self {
            search_method: method,
        }
    }
}

impl From<Search> for Event {
    fn from(it: Search) -> Self {
        Message::Search(it).into()
    }
}
