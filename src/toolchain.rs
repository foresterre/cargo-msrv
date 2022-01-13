use rust_releases::semver;

#[derive(Debug)]
pub struct ToolchainSpec<'v> {
    version: &'v semver::Version,
    spec: String,
}

impl<'v> ToolchainSpec<'v> {
    pub fn new(target: &str, version: &'v semver::Version) -> Self {
        Self {
            spec: format!("{}-{}", version, target),
            version,
        }
    }

    pub fn spec(&self) -> &str {
        &self.spec
    }

    pub fn version(&self) -> &semver::Version {
        self.version
    }

    pub fn to_owned(&self) -> OwnedToolchainSpec {
        OwnedToolchainSpec {
            version: self.version.clone(),
            spec: self.spec.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OwnedToolchainSpec {
    version: semver::Version,
    spec: String,
}

impl OwnedToolchainSpec {
    pub fn new(target: &str, version: semver::Version) -> Self {
        Self {
            spec: format!("{}-{}", version, target),
            version,
        }
    }

    pub fn spec(&self) -> &str {
        &self.spec
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }
}
