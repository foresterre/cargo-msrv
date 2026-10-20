use crate::{Event, Message};
use cargo_msrv_context::{EnvironmentContext, Project, SelectedPackage};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContextFinalized {
    pub project: ProjectMode,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum ProjectMode {
    Bare,
    Cargo {
        workspace: bool,
        packages: Vec<SelectedPackage>,
    },
}

impl ContextFinalized {
    pub fn new(environment: &EnvironmentContext) -> Self {
        let project = match &environment.project {
            Project::Bare => ProjectMode::Bare,
            Project::Cargo(project) => ProjectMode::Cargo {
                workspace: project.is_workspace(),
                packages: project.packages().map(SelectedPackage::from).collect(),
            },
        };

        Self { project }
    }

    pub fn packages(&self) -> &[SelectedPackage] {
        match &self.project {
            ProjectMode::Bare => &[],
            ProjectMode::Cargo { packages, .. } => packages,
        }
    }
}

impl From<ContextFinalized> for Event {
    fn from(it: ContextFinalized) -> Self {
        Message::ContextFinalized(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestReporterWrapper;
    use camino::Utf8PathBuf;
    use cargo_msrv_context::types::Edition;
    use cargo_msrv_context::{CargoProject, Package};
    use storyteller::EventReporter;

    fn package(name: &str) -> Package {
        Package {
            name: name.to_string(),
            manifest_path: Utf8PathBuf::from(format!("{name}/Cargo.toml")),
            rust_version: None,
            edition: Edition::Edition2021,
        }
    }

    fn selected(name: &str) -> SelectedPackage {
        SelectedPackage::from(&package(name))
    }

    fn cargo_environment(is_workspace: bool, packages: &[&str]) -> EnvironmentContext {
        let packages = packages.iter().map(|name| package(name)).collect();

        EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("ws"),
            project: Project::Cargo(
                CargoProject::new(Utf8PathBuf::from("ws"), is_workspace, packages).unwrap(),
            ),
        }
    }

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = ContextFinalized::new(&cargo_environment(true, &["a"]));

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::ContextFinalized(event))]
        );
    }

    #[yare::parameterized(
        single_package = { false, &["a"] },
        workspace = { true, &["a", "b"] },
    )]
    fn cargo_mode(workspace: bool, names: &[&str]) {
        let event = ContextFinalized::new(&cargo_environment(workspace, names));
        let packages = names.iter().map(|name| selected(name)).collect::<Vec<_>>();

        assert_eq!(event.packages(), packages.as_slice());
        assert_eq!(
            event.project,
            ProjectMode::Cargo {
                workspace,
                packages
            }
        );
    }

    #[test]
    fn bare_mode() {
        let environment = EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("ws"),
            project: Project::Bare,
        };

        let event = ContextFinalized::new(&environment);

        assert_eq!(event.project, ProjectMode::Bare);
        assert!(event.packages().is_empty());
    }

    #[yare::parameterized(
        bare = {
            ProjectMode::Bare,
            r#"{"type":"context_finalized","project":{"mode":"bare"}}"#
        },
        single_package = {
            ProjectMode::Cargo { workspace: false, packages: vec![selected("a")] },
            r#"{"type":"context_finalized","project":{"mode":"cargo","workspace":false,"packages":[{"name":"a","path":"a/Cargo.toml"}]}}"#
        },
        workspace = {
            ProjectMode::Cargo { workspace: true, packages: vec![selected("a"), selected("b")] },
            r#"{"type":"context_finalized","project":{"mode":"cargo","workspace":true,"packages":[{"name":"a","path":"a/Cargo.toml"},{"name":"b","path":"b/Cargo.toml"}]}}"#
        },
    )]
    fn serialized(project: ProjectMode, expected: &str) {
        let event: Event = ContextFinalized { project }.into();

        assert_eq!(serde_json::to_string(&event).unwrap(), expected);
    }
}
