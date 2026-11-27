use crate::state::{Running, Unexpected};
use crate::sub_command::{CompatibilityChecks, SubCommand};
use cargo_msrv_reporter::SubcommandResult;

pub struct Verify;

impl CompatibilityChecks for Verify {}

impl SubCommand for Verify {
    const SPINNER: bool = true;

    fn result(run: &mut Running<Self>, result: &SubcommandResult) -> Result<(), Unexpected> {
        match result {
            SubcommandResult::Verify(it) => {
                let version = it.toolchain().version();
                run.summary()
                    .verified(it.package(), version, it.is_compatible());
                Ok(())
            }
            SubcommandResult::Find(_)
            | SubcommandResult::List(_)
            | SubcommandResult::Set(_)
            | SubcommandResult::Show(_) => Err(Unexpected),
        }
    }
}
