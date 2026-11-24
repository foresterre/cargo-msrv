use crate::dependency_graph::DependencyGraph;
use camino::Utf8Path;
use cargo_metadata::MetadataCommand;

pub trait DependencyResolver {
    fn resolve(&self) -> Result<DependencyGraph, CargoMetadataResolveError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CargoMetadataResolveError {
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),
}

pub struct CargoMetadataResolver {
    metadata_command: MetadataCommand,
}

impl CargoMetadataResolver {
    pub fn from_manifest_path(path: &Utf8Path) -> Self {
        let mut metadata_command = MetadataCommand::new();
        metadata_command.manifest_path(path);

        Self { metadata_command }
    }
}

impl DependencyResolver for CargoMetadataResolver {
    fn resolve(&self) -> Result<DependencyGraph, CargoMetadataResolveError> {
        let result = self.metadata_command.exec()?;

        if let Some(dependencies) = result.resolve {
            let node_alloc = dependencies.nodes.len();
            let mut graph = DependencyGraph::with_capacity(node_alloc);

            build_package_graph(&mut graph, result.packages, dependencies.nodes);

            Ok(graph)
        } else {
            Ok(DependencyGraph::empty())
        }
    }
}

/// Builds a package graph from  1) a set of packages and 2) a given dependency graph.
fn build_package_graph<Ip, Id>(graph: &mut DependencyGraph, packages: Ip, dependencies: Id)
where
    Ip: IntoIterator<Item = cargo_metadata::Package>,
    Id: IntoIterator<Item = cargo_metadata::Node>,
{
    // Add nodes to the petgraph
    for package in packages {
        graph.add_package(package);
    }

    for dependency in dependencies {
        for child in dependency.deps {
            use cargo_metadata::DependencyKind;
            // do not include dev-dependencies
            // you need normal and build dependencies to build crates, but not dev
            if child
                .dep_kinds
                .iter()
                .all(|k| k.kind == DependencyKind::Normal || k.kind == DependencyKind::Build)
            {
                let child = graph.index()[&child.pkg];
                let ancestor = graph.index()[&dependency.id];

                // add link
                graph.add_dependency(ancestor, child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    fn package(dir: &TempDir, name: &str, dependencies: &str) {
        dir.child(name)
            .child("Cargo.toml")
            .write_str(&format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{dependencies}"
            ))
            .unwrap();
        dir.child(name).child("src/lib.rs").touch().unwrap();
    }

    fn virtual_workspace() -> TempDir {
        let dir = TempDir::new().unwrap();
        dir.child("Cargo.toml")
            .write_str("[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n")
            .unwrap();
        package(&dir, "a", "b = { path = \"../b\" }\n");
        package(&dir, "b", "");
        dir
    }

    fn root(dir: &TempDir) -> camino::Utf8PathBuf {
        let path = dunce::canonicalize(dir.path()).unwrap();
        camino::Utf8PathBuf::from_path_buf(path).unwrap()
    }

    fn resolve(dir: &TempDir) -> DependencyGraph {
        CargoMetadataResolver::from_manifest_path(&root(dir).join("Cargo.toml"))
            .resolve()
            .unwrap()
    }

    fn manifest_of(dir: &TempDir, name: &str) -> camino::Utf8PathBuf {
        root(dir).join(name).join("Cargo.toml")
    }

    #[test]
    fn virtual_workspace_has_every_member() {
        let dir = virtual_workspace();
        let graph = resolve(&dir);

        assert!(
            graph
                .find_by_manifest_path(&manifest_of(&dir, "a"))
                .is_some()
        );
        assert!(
            graph
                .find_by_manifest_path(&manifest_of(&dir, "b"))
                .is_some()
        );
    }

    #[test]
    fn path_dependency_is_an_edge() {
        let dir = virtual_workspace();
        let graph = resolve(&dir);

        let a = graph
            .find_by_manifest_path(&manifest_of(&dir, "a"))
            .unwrap();
        let dependencies = graph
            .packages()
            .neighbors_directed(a.into(), petgraph::Direction::Outgoing)
            .map(|index| graph.packages()[index].name.to_string())
            .collect::<Vec<_>>();

        assert_eq!(dependencies, ["b"]);
    }

    #[test]
    fn unknown_manifest_path() {
        let dir = virtual_workspace();
        let graph = resolve(&dir);

        assert!(
            graph
                .find_by_manifest_path(Utf8Path::new("/does/not/exist/Cargo.toml"))
                .is_none()
        );
    }

    #[test]
    fn graphs_of_the_same_workspace_are_equal() {
        let dir = virtual_workspace();

        assert_eq!(resolve(&dir), resolve(&dir));
    }

    #[test]
    fn graphs_of_different_workspaces_are_not_equal() {
        let dir = virtual_workspace();
        let other = TempDir::new().unwrap();
        other
            .child("Cargo.toml")
            .write_str("[workspace]\nmembers = [\"b\"]\nresolver = \"2\"\n")
            .unwrap();
        package(&other, "b", "");

        assert_ne!(resolve(&dir), resolve(&other));
    }
}
