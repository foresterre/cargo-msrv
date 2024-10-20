use cargo_metadata::{Package, PackageId};
use petgraph::visit::Dfs;
use std::collections::HashMap;

pub(crate) mod resolver;

type PackageGraphIndex = usize;
// NB: stable graph because we need our DependencyGraph::index to be able to bridge between id's
//  even after removals, which we do to remove dev- and build dependencies.
type PackageGraph =
    petgraph::stable_graph::StableDiGraph<cargo_metadata::Package, (), PackageGraphIndex>;

/// A graph of dependencies from a designated root crate
///
/// Why a graph instead of simply a set of packages?
/// To find the MSRV, all we need is the set, however, to locate where that dependency originates from,
/// it is useful to have a graph.
#[derive(Clone, Debug)]
pub struct DependencyGraph {
    // Useful to translate between the packages known to cargo_metadata and our petgraph.
    index: HashMap<PackageId, PackageGraphIndex>,
    // A directed graph of packages
    packages: PackageGraph,
    // The root crate is the crate we're creating the dependency graph for.
    root_crate: PackageId,
}

impl DependencyGraph {
    pub fn empty(root_crate: PackageId) -> Self {
        Self {
            index: HashMap::default(),
            packages: PackageGraph::with_capacity(0, 0),
            root_crate,
        }
    }

    pub fn with_capacity(root_crate: PackageId, cap: usize) -> Self {
        Self {
            index: HashMap::default(),
            packages: PackageGraph::with_capacity(cap, cap),
            root_crate,
        }
    }

    pub fn index(&self) -> &HashMap<PackageId, PackageGraphIndex> {
        &self.index
    }

    pub fn packages(&self) -> &PackageGraph {
        &self.packages
    }

    pub fn root_crate(&self) -> &PackageId {
        &self.root_crate
    }
}

impl PartialEq for DependencyGraph {
    fn eq(&self, other: &Self) -> bool {
        fn packages(graph: &DependencyGraph) -> Vec<&Package> {
            let package_id = graph.root_crate();
            let root_index = graph.index()[package_id].into();

            let mut res = Vec::with_capacity(graph.packages().node_count());

            while let Some(dep) = Dfs::new(graph.packages(), root_index).next(graph.packages()) {
                let package = &graph.packages()[dep];
                res.push(package);
            }

            res
        }

        self.root_crate() == other.root_crate() && packages(self) == packages(other)
    }
}
