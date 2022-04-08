use crate::check::Check;
use crate::outcome::Outcome;
use crate::semver::Version;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, ModeIntent, Output, ProgressAction, TResult};
use rust_releases::semver;
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq)]
pub enum Record {
    FetchIndex,
    InstallingToolchain(semver::Version),
    CheckToolchain(semver::Version),

    StepComplete,

    CmdWasSuccess,
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
    fn mode(&self, _mode: ModeIntent) {}

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

    fn complete_step(&self, _version: &Version, _success: bool) {
        self.log.borrow_mut().push(Record::StepComplete);
    }

    fn finish_success(&self, _mode: ModeIntent, version: Option<&Version>) {
        if version.is_some() {
            self.log.borrow_mut().push(Record::CmdWasSuccess);
        }
    }

    fn finish_failure(&self, _mode: ModeIntent, _cmd: Option<&str>) {
        self.log.borrow_mut().push(Record::CmdWasFailure);
    }

    fn write_line(&self, _content: &str) {}
}

pub struct TestRunner {
    accept_versions: HashSet<semver::Version>,
}

impl TestRunner {
    pub fn with_ok<'v, T: IntoIterator<Item = &'v Version>>(iter: T) -> Self {
        Self {
            accept_versions: iter.into_iter().cloned().collect(),
        }
    }
}

impl Check for TestRunner {
    fn check(&self, config: &Config, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        let v = toolchain.version();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Outcome::new_success(OwnedToolchainSpec::new(
                v,
                config.target(),
            )))
        } else {
            Ok(Outcome::new_failure(
                OwnedToolchainSpec::new(v, config.target()),
                "f".to_string(),
            ))
        }
    }
}
