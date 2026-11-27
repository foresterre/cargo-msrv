use crate::state::{Running, Unexpected};
use crate::sub_command::{CompatibilityChecks, Set, SubCommand};
use crate::view::Human;
use cargo_msrv_reporter::SubcommandResult;

mod view;

pub struct Find;

impl CompatibilityChecks for Find {}

impl SubCommand for Find {
    const SPINNER: bool = true;

    fn result(run: &mut Running<Self>, result: &SubcommandResult) -> Result<(), Unexpected> {
        match result {
            SubcommandResult::Find(it) => {
                run.line(Human(it));
                run.summary().msrv(it.package(), it.msrv());
                Ok(())
            }
            SubcommandResult::Set(it) => {
                Set::report(run, it);
                Ok(())
            }
            SubcommandResult::List(_) | SubcommandResult::Show(_) | SubcommandResult::Verify(_) => {
                Err(Unexpected)
            }
        }
    }
}
