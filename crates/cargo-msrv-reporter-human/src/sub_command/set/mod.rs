use crate::state::{Running, Unexpected};
use crate::sub_command::{ResultOnly, SubCommand};
use crate::view::Human;
use cargo_msrv_reporter::SubcommandResult;
use cargo_msrv_reporter::event::SetResult;

mod view;

pub struct Set;

impl ResultOnly for Set {}

impl SubCommand for Set {
    const SPINNER: bool = false;

    fn result(run: &mut Running<Self>, result: &SubcommandResult) -> Result<(), Unexpected> {
        match result {
            SubcommandResult::Set(it) => {
                Set::report(run, it);
                Ok(())
            }
            SubcommandResult::Find(_)
            | SubcommandResult::List(_)
            | SubcommandResult::Show(_)
            | SubcommandResult::Verify(_) => Err(Unexpected),
        }
    }
}

impl Set {
    pub fn report<C>(run: &mut Running<C>, it: &SetResult) {
        run.line(Human(it));
        run.summary().written(it.package(), it.version());
    }
}
