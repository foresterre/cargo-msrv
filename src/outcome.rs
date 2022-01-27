use rust_releases::semver;

#[derive(Clone, Debug)]
pub struct Outcome {
    pub(crate) status: Status,
    // toolchain specifier
    toolchain_spec: String,
    // checked Rust version
    version: semver::Version,
}

impl Outcome {
    pub fn new(status: Status, toolchain_spec: String, version: semver::Version) -> Self {
        Self {
            status,
            toolchain_spec,
            version,
        }
    }

    pub fn is_success(&self) -> bool {
        match self.status {
            Status::Success => true,
            Status::Failure(_) => false,
        }
    }

    pub fn status(&self) -> Status {
        self.status.clone()
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    pub fn toolchain_spec(&self) -> &str {
        &self.toolchain_spec
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Success,
    Failure(String),
}
