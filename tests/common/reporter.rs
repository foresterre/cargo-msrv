// Copied from src/testing.rs for integration and end-to-end testing
// To do: Merge them back together in a testing dev-dep crate:
// * Requires: traits for Check, Output etc. to be separated to a library crate as
//      well.

use cargo_msrv::config::Action;
use cargo_msrv::reporter::{Output, ProgressAction};
use rust_releases::semver;
use rust_releases::semver::Version;
use std::cell::{Cell, Ref, RefCell};
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq)]
pub enum Record {
    FetchIndex,
    InstallingToolchain(semver::Version),
    CheckToolchain(semver::Version),

    StepComplete {
        version: semver::Version,
        success: bool,
    },

    CmdWasSuccess,
    CmdWasSuccessWithVersion(semver::Version),
    CmdWasFailure,
}

#[derive(Debug)]
pub struct TestResultReporter {
    steps_left: Cell<u64>,
    log: Rc<RefCell<Vec<Record>>>,
}

impl Default for TestResultReporter {
    fn default() -> Self {
        Self {
            steps_left: Cell::new(0),
            log: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl TestResultReporter {
    #[allow(unused)]
    pub fn total_progress(&self) -> u64 {
        self.steps_left.get()
    }

    pub fn log(&self) -> Ref<'_, Vec<Record>> {
        self.log.borrow()
    }
}

impl Output for TestResultReporter {
    fn mode(&self, _mode: Action) {}

    fn set_steps(&self, steps: u64) {
        self.steps_left.replace(steps);
    }

    fn progress(&self, action: ProgressAction) {
        let record = match action {
            ProgressAction::Checking(v) => Record::CheckToolchain(v.clone()),
            ProgressAction::FetchingIndex => Record::FetchIndex,
            ProgressAction::Installing(v) => Record::InstallingToolchain(v.clone()),
        };

        self.log.borrow_mut().push(record);
    }

    fn complete_step(&self, version: &Version, success: bool) {
        self.log.borrow_mut().push(Record::StepComplete {
            version: version.clone(),
            success,
        });
    }

    fn finish_success(&self, _mode: Action, version: Option<&Version>) {
        if let Some(v) = version {
            self.log
                .borrow_mut()
                .push(Record::CmdWasSuccessWithVersion(v.clone()));
        } else {
            self.log.borrow_mut().push(Record::CmdWasSuccess);
        }
    }

    fn finish_failure(&self, _mode: Action, _cmd: Option<&str>) {
        self.log.borrow_mut().push(Record::CmdWasFailure);
    }

    fn write_line(&self, _content: &str) {}
}
