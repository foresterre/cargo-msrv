use once_cell::sync::OnceCell;
use rust_releases::semver;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ToolchainSpec<'spec> {
    version: &'spec semver::Version,
    target: &'spec str,
    spec: OnceCell<String>,
}

impl<'spec> ToolchainSpec<'spec> {
    pub fn new(version: &'spec semver::Version, target: &'spec str) -> Self {
        Self {
            version,
            target,
            spec: OnceCell::new(),
        }
    }

    pub fn spec(&self) -> &str {
        self.spec
            .get_or_init(|| make_toolchain_spec(self.version, self.target))
    }

    pub fn version(&self) -> &semver::Version {
        self.version
    }

    pub fn to_owned(&self) -> OwnedToolchainSpec {
        OwnedToolchainSpec {
            version: self.version.clone(),
            target: self.target.to_string(),
            spec: self.spec.clone(),
        }
    }
}

impl<'r> From<ToolchainSpec<'r>> for OwnedToolchainSpec {
    fn from(spec: ToolchainSpec<'r>) -> Self {
        spec.to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OwnedToolchainSpec {
    version: semver::Version,
    target: String,
    #[serde(skip)]
    spec: OnceCell<String>,
}

impl OwnedToolchainSpec {
    pub fn new(version: &semver::Version, target: &str) -> Self {
        Self {
            version: version.clone(),
            target: target.to_string(),
            spec: OnceCell::new(),
        }
    }

    pub fn spec(&self) -> &str {
        self.spec
            .get_or_init(|| make_toolchain_spec(&self.version, &self.target))
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

impl std::fmt::Display for OwnedToolchainSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.spec()))
    }
}

fn make_toolchain_spec(version: &semver::Version, target: &str) -> String {
    format!("{}-{}", version, target)
}
