pub mod resolver;

use camino::Utf8Path;
use cargo_metadata::{Package, PackageId};
use std::collections::HashMap;

pub type PackageGraphIndex = usize;
// NB: stable graph because we need our DependencyGraph::index to be able to bridge between id's
//  even after removals, which we do to remove dev- and build dependencies.
type PackageGraph = petgraph::stable_graph::StableDiGraph<Package, (), PackageGraphIndex>;

/// A graph of the dependencies of the packages in a workspace
///
/// Why a graph instead of simply a set of packages?
/// To find the MSRV, all we need is the set, however, to locate where that dependency originates from,
/// it is useful to have a graph.
#[derive(Clone, Debug, Default)]
pub struct DependencyGraph {
    // Useful to translate between the packages known to cargo_metadata and our petgraph.
    index: HashMap<PackageId, PackageGraphIndex>,
    // A directed graph of packages
    packages: PackageGraph,
}

impl DependencyGraph {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            index: HashMap::default(),
            packages: PackageGraph::with_capacity(cap, cap),
        }
    }

    pub fn add_package(&mut self, package: Package) {
        let package_id = package.id.clone();
        let node_index = self.packages.add_node(package);
        let _ = self.index.insert(package_id, node_index.index());
    }

    pub fn add_dependency(&mut self, ancestor: PackageGraphIndex, child: PackageGraphIndex) {
        self.packages.add_edge(ancestor.into(), child.into(), ());
    }

    pub fn index(&self) -> &HashMap<PackageId, PackageGraphIndex> {
        &self.index
    }

    pub fn packages(&self) -> &PackageGraph {
        &self.packages
    }

    /// Find the package defined by the Cargo manifest at the given path
    pub fn find_by_manifest_path(&self, path: &Utf8Path) -> Option<PackageGraphIndex> {
        self.packages
            .node_indices()
            .find(|&index| self.packages[index].manifest_path == path)
            .map(|index| index.index())
    }
}

impl PartialEq for DependencyGraph {
    fn eq(&self, other: &Self) -> bool {
        Topology::new(self) == Topology::new(other)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Edge<'g>(&'g PackageId, &'g PackageId);

#[derive(PartialEq, Eq)]
struct Topology<'g>(Vec<&'g PackageId>, Vec<Edge<'g>>);

impl<'g> Topology<'g> {
    fn new(graph: &'g DependencyGraph) -> Self {
        let packages = graph.packages();

        let mut ids = packages
            .node_weights()
            .map(|package| &package.id)
            .collect::<Vec<_>>();
        ids.sort();

        let mut edges = packages
            .edge_indices()
            .filter_map(|edge| packages.edge_endpoints(edge))
            .map(|(from, to)| Edge(&packages[from].id, &packages[to].id))
            .collect::<Vec<_>>();
        edges.sort();

        Self(ids, edges)
    }
}
