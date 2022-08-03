use crate::reporter::event::{FindResult, ListResult, SetResult, ShowResult, VerifyResult};
use crate::reporter::Message;
use crate::Event;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum SubcommandResult {
    Find(FindResult),
    List(ListResult),
    Set(SetResult),
    Show(ShowResult),
    Verify(VerifyResult),
}

impl From<SubcommandResult> for Event {
    fn from(this: SubcommandResult) -> Self {
        Message::SubcommandResult(this).into()
    }
}
