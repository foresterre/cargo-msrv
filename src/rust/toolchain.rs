use rust_releases::semver;
use std::sync::OnceLock;

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Toolchain {
    version: semver::Version,
    target: &'static str,
    components: &'static [&'static str],
    #[serde(skip)]
    spec: OnceLock<String>,
}

impl Toolchain {
    pub fn new(
        version: semver::Version,
        target: &'static str,
        components: &'static [&'static str],
    ) -> Self {
        Self {
            version,
            target,
            components,
            spec: OnceLock::new(),
        }
    }

    pub fn spec(&self) -> &str {
        self.spec
            .get_or_init(|| make_toolchain_spec(&self.version, self.target))
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    pub fn components(&self) -> &'static [&'static str] {
        self.components
    }

    pub fn target(&self) -> &str {
        self.target
    }
}

impl std::fmt::Display for Toolchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.spec()))
    }
}

fn make_toolchain_spec(version: &semver::Version, target: &str) -> String {
    format!("{}-{}", version, target)
}

#[cfg(test)]
mod tests_toolchain_spec {
    use super::*;

    #[test]
    fn get_spec() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        assert_eq!(toolchain.spec(), "1.2.3-x");
    }

    #[test]
    fn get_version() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        assert_eq!(toolchain.version(), &semver::Version::new(1, 2, 3));
    }

    #[test]
    fn get_target() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        assert_eq!(toolchain.target(), "x");
    }

    #[test]
    fn get_components() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &["hello", "chris!"]);

        assert_eq!(toolchain.components(), &["hello", "chris!"]);
    }
}

#[cfg(test)]
mod tests_make_toolchain_spec {
    use super::*;

    #[test]
    fn display() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "y", &[]);

        let spec = format!("{}", toolchain);

        assert_eq!(spec, "1.2.3-y");
    }

    #[test]
    fn make_spec() {
        let version = semver::Version::new(1, 2, 3);
        let spec = make_toolchain_spec(&version, "y");

        assert_eq!(spec, "1.2.3-y");
    }

    #[test]
    fn display_ignores_components() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "y", &["to", "be", "ignored"]);

        let spec = format!("{}", toolchain);

        assert_eq!(spec, "1.2.3-y");
    }
}
