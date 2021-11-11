use cargo_metadata::PackageId;
use std::collections::HashMap;

pub(crate) mod formatter;
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
#[derive(Debug)]
pub(crate) struct DependencyGraph {
    // Useful to translate between the packages known to cargo_metadata and our petgraph.
    index: HashMap<cargo_metadata::PackageId, PackageGraphIndex>,
    // A directed graph of packages
    packages: PackageGraph,
    // The root crate is the crate we're creating the dependency graph for.
    root_crate: cargo_metadata::PackageId,
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
}
