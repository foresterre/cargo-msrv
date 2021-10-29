use crate::dependencies::formatter::{
    format_version, get_package_metadata_msrv, parse_manifest_workaround,
};
use crate::dependencies::DependencyGraph;
use cargo_metadata::Package;
use petgraph::visit::Bfs;
use std::collections::BTreeMap;
use std::fmt::Formatter;

/// Displays the dependencies of the project as a table, sorted by their specified MSRV.
///
/// For example:
///
/// ```md
/// | MSRV   | Crates |
/// | 1.56.0 | some-dep, some-other-dep |
/// | 1.12.0 | some |
///
/// MSRV for {crate}: { min(MSRVs) }
/// ```
pub(crate) struct ByMSRVFormatter {
    graph: DependencyGraph,
}

impl ByMSRVFormatter {
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }
}

impl std::fmt::Display for ByMSRVFormatter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Table of dependencies
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::*;

        let mut table = Table::new();

        // MSRV = package.rust_version /fallback package.metadata.msrv
        // Dependency = {name of package}
        // Type = { direct | transitive }
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["MSRV", "Dependency"]);

        let dependency_graph = &self.graph;

        let root = &dependency_graph.root_crate;
        let root = dependency_graph.index[root];
        let graph = &dependency_graph.packages;

        // let mut direct_deps = graph.neighbors_directed(root.into(), Direction::Outgoing);
        let mut bfs = Bfs::new(&graph, root.into());

        // todo: sorting
        use crate::semver;
        let mut version_map: BTreeMap<Option<semver::Version>, Vec<&Package>> = BTreeMap::new();

        while let Some(nx) = bfs.next(&graph) {
            let package = &graph[nx];

            let msrv = package
                .rust_version
                .to_owned()
                .map(|req| {
                    let comparator = &req.comparators[0];
                    crate::semver::Version::new(
                        comparator.major,
                        comparator.minor.unwrap_or_default(),
                        comparator.patch.unwrap_or_default(),
                    )
                })
                .or_else(|| get_package_metadata_msrv(package))
                .or_else(|| parse_manifest_workaround(package.manifest_path.as_path())); // todo: add last one as option to config

            version_map.entry(msrv).or_default().push(package);
        }

        for (version, packages) in version_map {
            table.add_row(vec![
                Cell::new(format_version(version.as_ref())),
                Cell::new(format_packages(&packages)),
            ]);
        }

        table.fmt(f)
    }
}

fn format_packages(dependencies: &[&cargo_metadata::Package]) -> String {
    dependencies
        .iter()
        .map(|pkg| pkg.name.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
