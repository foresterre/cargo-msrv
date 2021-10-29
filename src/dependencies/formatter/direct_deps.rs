use crate::dependencies::formatter::{
    format_version, get_package_metadata_msrv, parse_manifest_workaround,
};
use crate::dependencies::DependencyGraph;

pub(crate) struct DirectDependenciesFormatter {
    graph: DependencyGraph,
}

impl DirectDependenciesFormatter {
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }
}

impl std::fmt::Display for DirectDependenciesFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Table of dependencies
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::*;

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_table_width(120) // fallback for ContentArrangement::Dynamic
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Dependency", "Version", "MSRV", "Depends on"]);

        // Table
        // Direct dependencies
        // e.g.
        //
        //
        //
        // | json | {version} | {MSRV} | { transitive deps with color = red if lowest version,
        //                                  and color = orange if lower version than own MSRV }
        let dependency_graph = &self.graph;

        let root = &dependency_graph.root_crate;
        let root = dependency_graph.index[root];

        let graph = &dependency_graph.packages;

        let neighbors = graph.neighbors_directed(root.into(), petgraph::Direction::Outgoing);

        for dep in neighbors {
            let package = &graph[dep];

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

            table.add_row(vec![
                Cell::new(&package.name),
                Cell::new(&package.version),
                Cell::new(format_version(msrv.as_ref())),
                Cell::new(format_dependencies(&package.dependencies)),
            ]);
        }

        table.fmt(f)
    }
}

fn format_dependencies(dependencies: &[cargo_metadata::Dependency]) -> String {
    dependencies
        .iter()
        .map(|dep| dep.name.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
