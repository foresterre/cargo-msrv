use crate::context::error::{Error, PackageWithoutMsrv, TResult};
use crate::context::{
    CheckCommandContext, EnvironmentContext, Package, Project, RustReleasesContext,
    ToolchainContext,
};
use camino::Utf8PathBuf;
use version_number::Version;

#[derive(Debug)]
pub struct VerifyContext {
    pub checks: Vec<VerifyCheck>,

    /// Ignore the lockfile for the MSRV verification
    pub ignore_lockfile: bool,

    /// Don't print the result of compatibility check
    pub no_check_feedback: bool,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for custom checks to be used with rustup
    pub check_cmd: CheckCommandContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}

#[derive(Clone, Debug)]
pub struct VerifyCheck {
    pub package: Option<Package>,
    pub rust_version: RustVersion,
}

impl VerifyCheck {
    pub fn resolve(
        rust_version: Option<&Version>,
        check_cmd: &CheckCommandContext,
        env: &EnvironmentContext,
    ) -> TResult<Vec<Self>> {
        if rust_version.is_none() {
            let without_msrv = packages_without_msrv(env);

            if !without_msrv.is_empty() {
                return Err(Error::NoMSRVKeyInCargoToml(
                    without_msrv
                        .into_iter()
                        .map(PackageWithoutMsrv::from)
                        .collect(),
                ));
            }
        }

        env.packages_to_check(check_cmd)
            .into_iter()
            .map(|package| {
                let rust_version = match (rust_version, package) {
                    (Some(v), _) => RustVersion::from_arg(v.clone()),
                    (None, Some(package)) => RustVersion::from_package(package)?,
                    (None, None) => RustVersion::from_project(env)?,
                };

                Ok(Self {
                    package: package.cloned(),
                    rust_version,
                })
            })
            .collect()
    }
}

fn packages_without_msrv(env: &EnvironmentContext) -> Vec<&Package> {
    match &env.project {
        Project::Cargo(project) => project
            .packages()
            .filter(|package| package.rust_version.is_none())
            .collect(),
        Project::Bare => Vec::new(),
    }
}

impl From<&Package> for PackageWithoutMsrv {
    fn from(package: &Package) -> Self {
        Self {
            name: package.name.clone(),
            manifest_path: package.manifest_path.clone(),
        }
    }
}

/// A combination of a bare (two- or three component) Rust version and the source which was used to
/// locate this version.
#[derive(Clone, Debug)]
pub struct RustVersion {
    rust_version: Version,
    source: RustVersionSource,
}

impl RustVersion {
    pub fn from_arg(rust_version: Version) -> Self {
        Self {
            rust_version,
            source: RustVersionSource::Arg,
        }
    }

    pub fn from_package(package: &Package) -> TResult<Self> {
        package
            .rust_version
            .clone()
            .ok_or_else(|| Error::NoMSRVKeyInCargoToml(vec![PackageWithoutMsrv::from(package)]))
            .map(|rust_version| RustVersion {
                rust_version,
                source: RustVersionSource::Manifest(package.manifest_path.clone()),
            })
    }

    pub fn from_project(env: &EnvironmentContext) -> TResult<Self> {
        let Project::Cargo(project) = &env.project else {
            return Err(Error::NoCargoManifest(env.manifest()));
        };

        let first = Self::from_package(&project.first)?;

        if project.rest.is_empty() {
            return Ok(first);
        }

        for package in &project.rest {
            if Self::from_package(package)?.rust_version != first.rust_version {
                return Err(Error::MixedMSRVs);
            }
        }

        Ok(RustVersion {
            rust_version: first.rust_version,
            source: RustVersionSource::SelectedPackages,
        })
    }

    /// Get the bare (two- or three component) version specifying the Rust version.
    pub fn version(&self) -> &Version {
        &self.rust_version
    }

    /// Get the version and discard all else.
    pub fn into_version(self) -> Version {
        self.rust_version
    }

    /// Get the version and the source which was used to locate it.
    pub fn into_parts(self) -> (Version, RustVersionSource) {
        (self.rust_version, self.source)
    }
}

/// Source used to obtain a Rust version for the verifier.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RustVersionSource {
    #[error("as --rust-version argument")]
    Arg,

    #[error("as MSRV in the Cargo manifest located at '{0}'")]
    Manifest(Utf8PathBuf),

    #[error("as MSRV of every selected package")]
    SelectedPackages,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CargoProject;
    use crate::types::Edition;

    fn package(name: &str, rust_version: Option<Version>) -> Package {
        Package {
            name: name.to_string(),
            manifest_path: Utf8PathBuf::from(format!("{name}/Cargo.toml")),
            rust_version,
            edition: Edition::Edition2021,
        }
    }

    fn cargo_environment(packages: Vec<Package>) -> EnvironmentContext {
        EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("ws"),
            project: Project::Cargo(CargoProject::new(Utf8PathBuf::from("ws"), packages).unwrap()),
        }
    }

    fn check_cmd(custom: Option<Vec<String>>) -> CheckCommandContext {
        CheckCommandContext {
            cargo_features: None,
            cargo_all_features: false,
            cargo_no_default_features: false,
            rustup_command: custom,
        }
    }

    fn v(minor: u64) -> Version {
        Version::new_base_version(1, minor)
    }

    #[test]
    fn rust_version_without_manifest() {
        let root = Utf8PathBuf::from("bare");
        let env = EnvironmentContext {
            root_crate_path: root.clone(),
            project: Project::Bare,
        };

        let result = VerifyCheck::resolve(None, &check_cmd(None), &env);

        assert!(
            matches!(result, Err(Error::NoCargoManifest(path)) if path == root.join("Cargo.toml"))
        );
    }

    #[test]
    fn rust_version_arg_without_manifest() {
        let env = EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("bare"),
            project: Project::Bare,
        };

        let checks =
            VerifyCheck::resolve(Some(&v(60)), &check_cmd(Some(vec!["make".into()])), &env)
                .unwrap();

        assert_eq!(checks.len(), 1);
        assert!(checks[0].package.is_none());
        assert_eq!(checks[0].rust_version.version(), &v(60));
    }

    #[test]
    fn each_package_has_its_own_rust_version() {
        let env = cargo_environment(vec![package("a", Some(v(56))), package("b", Some(v(58)))]);

        let checks = VerifyCheck::resolve(None, &check_cmd(None), &env).unwrap();

        let versions = checks
            .iter()
            .map(|c| {
                (
                    c.package.as_ref().unwrap().name.as_str(),
                    c.rust_version.version(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(versions, [("a", &v(56)), ("b", &v(58))]);
    }

    #[test]
    fn rust_version_arg_applies_to_every_package() {
        let env = cargo_environment(vec![package("a", Some(v(56))), package("b", None)]);

        let checks = VerifyCheck::resolve(Some(&v(70)), &check_cmd(None), &env).unwrap();

        assert!(checks.iter().all(|c| c.rust_version.version() == &v(70)));
    }

    fn without_msrv(result: TResult<Vec<VerifyCheck>>) -> Vec<String> {
        match result {
            Err(Error::NoMSRVKeyInCargoToml(packages)) => {
                packages.into_iter().map(|p| p.name).collect()
            }
            other => panic!("expected NoMSRVKeyInCargoToml, got {other:?}"),
        }
    }

    #[test]
    fn package_without_rust_version() {
        let env = cargo_environment(vec![package("a", Some(v(56))), package("b", None)]);

        let result = VerifyCheck::resolve(None, &check_cmd(None), &env);

        assert_eq!(without_msrv(result), ["b"]);
    }

    #[test]
    fn every_package_without_rust_version_is_reported() {
        let env = cargo_environment(vec![
            package("a", None),
            package("b", Some(v(56))),
            package("c", None),
        ]);

        let result = VerifyCheck::resolve(None, &check_cmd(None), &env);

        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Unable to find key 'package.rust-version' (or 'package.metadata.msrv') for 2 packages: \
             'a' ('a/Cargo.toml'), 'c' ('c/Cargo.toml'). \
             Use '--rust-version <VERSION>' to verify them against a single version, \
             or leave them out with '--exclude <NAME>'."
        );
    }

    #[test]
    fn single_package_without_rust_version_message() {
        let env = cargo_environment(vec![package("a", None)]);

        let err = VerifyCheck::resolve(None, &check_cmd(None), &env).unwrap_err();

        assert_eq!(
            err.to_string(),
            "Unable to find key 'package.rust-version' (or 'package.metadata.msrv') in 'a/Cargo.toml'"
        );
    }

    #[test]
    fn custom_command_reports_every_package_without_rust_version() {
        let env = cargo_environment(vec![
            package("a", None),
            package("b", Some(v(56))),
            package("c", None),
        ]);

        let result = VerifyCheck::resolve(None, &check_cmd(Some(vec!["make".into()])), &env);

        assert_eq!(without_msrv(result), ["a", "c"]);
    }

    #[test]
    fn custom_command_with_same_rust_versions() {
        let env = cargo_environment(vec![package("a", Some(v(56))), package("b", Some(v(56)))]);

        let checks =
            VerifyCheck::resolve(None, &check_cmd(Some(vec!["make".into()])), &env).unwrap();

        assert_eq!(checks.len(), 1);
        assert!(checks[0].package.is_none());
        assert_eq!(checks[0].rust_version.version(), &v(56));
    }

    #[test]
    fn custom_command_with_mixed_rust_versions() {
        let env = cargo_environment(vec![package("a", Some(v(56))), package("b", Some(v(58)))]);

        let result = VerifyCheck::resolve(None, &check_cmd(Some(vec!["make".into()])), &env);

        assert!(matches!(result, Err(Error::MixedMSRVs)));
    }
}
