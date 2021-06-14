use crate::config::ModeIntent;
use rust_releases::semver;

pub mod json;
pub mod ui;

#[derive(Debug, Clone, Copy)]
pub enum ProgressAction {
    Installing,
    Checking,
}

pub trait Output {
    // Shows the mode in which cargo-msrv will operate
    fn mode(&self, mode: ModeIntent);

    // Sets the remaining amount of steps for the mode
    fn set_steps(&self, steps: u64);

    // Reports the currently running
    fn progress(&self, action: ProgressAction, version: &semver::Version);
    fn complete_step(&self, version: &semver::Version, success: bool);
    fn finish_success(&self, mode: ModeIntent, version: &semver::Version);
    fn finish_failure(&self, mode: ModeIntent, cmd: &str);
}

pub mod __private {
    use crate::config::ModeIntent;
    use crate::reporter::{Output, ProgressAction};
    use rust_releases::semver;

    /// This is meant to be used for testing
    pub struct NoOutput;

    impl Output for NoOutput {
        fn mode(&self, _action: ModeIntent) {}
        fn set_steps(&self, _steps: u64) {}
        fn progress(&self, _action: ProgressAction, _version: &semver::Version) {}
        fn complete_step(&self, _version: &semver::Version, _success: bool) {}
        fn finish_success(&self, _mode: ModeIntent, _version: &semver::Version) {}
        fn finish_failure(&self, _mode: ModeIntent, _cmd: &str) {}
    }
}
