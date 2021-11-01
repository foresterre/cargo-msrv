use crate::dependencies::formatter::{
    format_version, get_package_metadata_msrv, parse_manifest_workaround,
};
use crate::dependencies::DependencyGraph;
use crate::reporter::Output;
use cargo_metadata::Package;
use petgraph::visit::Bfs;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::marker::PhantomData;

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
pub(crate) struct ByMSRVFormatter<T: Output> {
    graph: DependencyGraph,
    output: PhantomData<T>,
}

impl<T: Output> ByMSRVFormatter<T> {
    pub fn new(graph: DependencyGraph) -> Self {
        Self {
            graph,
            output: PhantomData,
        }
    }
}

impl<T: Output> ByMSRVFormatter<T> {
    fn dependencies_by_msrv<Fi, Fg, B>(&self, init: Fi, f: Fg) -> B
    where
        Fi: FnOnce() -> B,
        Fg: Fn(&mut B, Values),
    {
        let mut out = init();

        let dependency_graph = &self.graph;

        let root = &dependency_graph.root_crate;
        let root = dependency_graph.index[root];
        let graph = &dependency_graph.packages;

        // let mut direct_deps = graph.neighbors_directed(root.into(), Direction::Outgoing);
        let mut bfs = Bfs::new(&graph, root.into());

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
            let values = Values {
                msrv: format_version(version.as_ref()),
                dependencies: packages.iter().map(|p| p.name.to_owned()).collect(),
            };

            f(&mut out, values);
        }

        out
    }
}

impl std::fmt::Display for ByMSRVFormatter<crate::reporter::ui::HumanPrinter<'_, '_>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Table of dependencies sorted by MSRV
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::*;

        let out = self.dependencies_by_msrv(
            || {
                let mut table = Table::new();

                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["MSRV", "Dependency"]);
                table
            },
            |acc, next| {
                acc.add_row(vec![
                    Cell::new(&next.msrv),
                    Cell::new(&next.dependencies.join(", ")),
                ]);
            },
        );

        out.fmt(f)
    }
}

impl std::fmt::Display for ByMSRVFormatter<crate::reporter::json::JsonPrinter<'_, '_>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Table of dependencies sorted by MSRV
        use json::object;

        let objects = self.dependencies_by_msrv(Vec::new, |acc, next| {
            acc.push(object! {
                "msrv": next.msrv,
                "dependencies": next.dependencies
            })
        });

        let json = object! {
            reason: "list",
            variant: crate::config::list::ORDERED_BY_MSRV,
            success: true,
            list: objects,
        };

        writeln!(f, "{}", json)
    }
}

struct Values {
    msrv: String,
    dependencies: Vec<String>,
}
