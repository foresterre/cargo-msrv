use crate::state::{Running, Unexpected};
use crate::sub_command::{ResultOnly, SubCommand};
use crate::view::Human;
use cargo_msrv_reporter::SubcommandResult;

mod view;

pub struct List;

impl ResultOnly for List {}

impl SubCommand for List {
    const SPINNER: bool = false;

    fn result(run: &mut Running<Self>, result: &SubcommandResult) -> Result<(), Unexpected> {
        match result {
            SubcommandResult::List(it) => {
                run.line(Human(it));
                Ok(())
            }
            SubcommandResult::Find(_)
            | SubcommandResult::Set(_)
            | SubcommandResult::Show(_)
            | SubcommandResult::Verify(_) => Err(Unexpected),
        }
    }
}
