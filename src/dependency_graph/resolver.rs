use crate::config::Config;
use crate::dependency_graph::DependencyGraph;
use crate::error::{CargoMSRVError, TResult};
use crate::paths::crate_root_folder;
use cargo_metadata::MetadataCommand;

pub(crate) trait DependencyResolver {
    fn resolve(&self) -> TResult<DependencyGraph>;
}

pub(crate) struct CargoMetadataResolver {
    metadata_command: MetadataCommand,
}

impl CargoMetadataResolver {
    pub fn try_from_config(config: &Config) -> TResult<Self> {
        let crate_root = crate_root_folder(config)?;

        let mut metadata_command = MetadataCommand::new();
        metadata_command.manifest_path(crate_root.join("Cargo.toml"));

        Ok(Self { metadata_command })
    }
}

impl DependencyResolver for CargoMetadataResolver {
    fn resolve(&self) -> TResult<DependencyGraph> {
        let result = self.metadata_command.exec()?;

        let our_crate = result
            .root_package()
            .ok_or(CargoMSRVError::NoCrateRootFound)
            .map(|pkg| pkg.id.clone())?;

        if let Some(dependencies) = result.resolve {
            let node_alloc = dependencies.nodes.len();
            let mut graph = DependencyGraph::with_capacity(our_crate, node_alloc);

            build_package_graph(
                &mut graph,
                result.packages.into_iter(),
                dependencies.nodes.into_iter(),
            );

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
        let package_id = package.id.clone();
        let node_index = graph.packages.add_node(package);
        let _ = graph.index.insert(package_id, node_index.index());
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
                let child = graph.index[&child.pkg];
                let ancestor = graph.index[&dependency.id];

                // add link
                graph.packages.add_edge(ancestor.into(), child.into(), ());
            }
        }
    }
}
