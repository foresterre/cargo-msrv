use crate::config;
use crate::config::ModeIntent;
use rust_releases::semver;
use rust_releases::semver::Version;

pub mod json;
pub mod ui;

#[derive(Debug, Clone, Copy)]
pub enum ProgressAction<'a> {
    Installing(&'a semver::Version),
    Checking(&'a semver::Version),
    FetchingIndex,
}

pub trait Output {
    // Shows the mode in which cargo-msrv will operate
    fn mode(&self, mode: ModeIntent);

    // Sets the remaining amount of steps for the mode
    fn set_steps(&self, steps: u64);

    // Reports the currently running
    fn progress(&self, action: ProgressAction);
    fn complete_step(&self, version: &semver::Version, success: bool);
    fn finish_success(&self, mode: ModeIntent, version: &semver::Version);
    fn finish_failure(&self, mode: ModeIntent, cmd: &str);
}

pub struct Reporter<'output> {
    output: Box<dyn Output + 'output>,
}

impl<'output> Reporter<'output> {
    fn new(output: Box<dyn Output + 'output>) -> Self {
        Self { output }
    }
}

impl<'output> Output for Reporter<'output> {
    fn mode(&self, mode: ModeIntent) {
        self.output.mode(mode)
    }

    fn set_steps(&self, steps: u64) {
        self.output.set_steps(steps)
    }

    fn progress(&self, action: ProgressAction) {
        self.output.progress(action)
    }

    fn complete_step(&self, version: &Version, success: bool) {
        self.output.complete_step(version, success)
    }

    fn finish_success(&self, mode: ModeIntent, version: &Version) {
        self.output.finish_success(mode, version)
    }

    fn finish_failure(&self, mode: ModeIntent, cmd: &str) {
        self.output.finish_failure(mode, cmd)
    }
}

pub struct ReporterBuilder<'s> {
    target: &'s str,
    cmd: &'s str,
    output_format: config::OutputFormat,
}

impl<'s> ReporterBuilder<'s> {
    pub fn new(target: &'s str, cmd: &'s str) -> Self {
        Self {
            target,
            cmd,
            output_format: Default::default(),
        }
    }

    pub fn output_format(mut self, output_format: config::OutputFormat) -> Self {
        self.output_format = output_format;
        self
    }

    pub fn build(self) -> Reporter<'s> {
        let boxed: Box<dyn Output> = match self.output_format {
            config::OutputFormat::Human => {
                Box::new(ui::HumanPrinter::new(1, self.target, self.cmd))
            }
            config::OutputFormat::Json => {
                Box::new(json::JsonPrinter::new(1, self.target, self.cmd))
            }
            config::OutputFormat::None => Box::new(__private::NoOutput),
        };

        Reporter::new(boxed)
    }
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
        fn progress(&self, _action: ProgressAction) {}
        fn complete_step(&self, _version: &semver::Version, _success: bool) {}
        fn finish_success(&self, _mode: ModeIntent, _version: &semver::Version) {}
        fn finish_failure(&self, _mode: ModeIntent, _cmd: &str) {}
    }
}
