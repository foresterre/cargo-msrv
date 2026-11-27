use crate::state::{Running, Unexpected};
use crate::sub_command::{ResultOnly, SubCommand};
use crate::view::Human;
use cargo_msrv_reporter::SubcommandResult;

mod view;

pub struct Show;

impl ResultOnly for Show {}

impl SubCommand for Show {
    const SPINNER: bool = false;

    fn result(run: &mut Running<Self>, result: &SubcommandResult) -> Result<(), Unexpected> {
        match result {
            SubcommandResult::Show(it) => {
                run.line(Human(it));
                run.summary().version(it.package(), it.version());
                Ok(())
            }
            SubcommandResult::Find(_)
            | SubcommandResult::List(_)
            | SubcommandResult::Set(_)
            | SubcommandResult::Verify(_) => Err(Unexpected),
        }
    }
}
