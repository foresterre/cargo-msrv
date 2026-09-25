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

    #[error("No crate root found for given crate")]
    NoCrateRootFound,
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

        let our_crate = result
            .root_package()
            .ok_or(CargoMetadataResolveError::NoCrateRootFound)
            .map(|pkg| pkg.id.clone())?;

        if let Some(dependencies) = result.resolve {
            let node_alloc = dependencies.nodes.len();
            let mut graph = DependencyGraph::with_capacity(our_crate, node_alloc);

            build_package_graph(&mut graph, result.packages, dependencies.nodes);

            Ok(graph)
        } else {
            Ok(DependencyGraph::empty(our_crate))
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
