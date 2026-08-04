use crate::Event;
use crate::Message;
use crate::event::{FindResult, ListResult, SetResult, ShowResult, VerifyResult};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "subcommand_id")]
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
