use crate::dependencies::formatter::{
    format_version, get_package_metadata_msrv, parse_manifest_workaround,
};
use crate::dependencies::DependencyGraph;
use crate::reporter::Output;
use std::marker::PhantomData;

pub(crate) struct DirectDependenciesFormatter<T: Output> {
    graph: DependencyGraph,
    output: PhantomData<T>,
}

impl<T: Output> DirectDependenciesFormatter<T> {
    pub fn new(graph: DependencyGraph) -> Self {
        Self {
            graph,
            output: PhantomData,
        }
    }
}

impl<T: Output> DirectDependenciesFormatter<T> {
    fn direct_dependencies_msrv<Fi, Fg, B>(&self, init: Fi, f: Fg) -> B
    where
        Fi: FnOnce() -> B,
        Fg: Fn(&mut B, Values<'_, '_>),
    {
        let mut out = init();

        let dependency_graph = &self.graph;

        let root = &dependency_graph.root_crate;
        let root = dependency_graph.index[root];

        let graph = &dependency_graph.packages;

        let neighbors = graph.neighbors_directed(root.into(), petgraph::Direction::Outgoing);

        for dep in neighbors {
            let package = &graph[dep];

            let msrv = package
                .rust_version
                .clone()
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

            let values = Values {
                name: &package.name,
                version: &package.version,
                msrv: format_version(msrv.as_ref()),
                dependencies: package
                    .dependencies
                    .iter()
                    .map(|d| d.name.clone())
                    .collect(),
            };

            f(&mut out, values);
        }

        out
    }
}

impl std::fmt::Display for DirectDependenciesFormatter<crate::reporter::ui::HumanPrinter<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Table of dependencies
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::{Cell, ContentArrangement, Table};

        let table = self.direct_dependencies_msrv(
            || {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_table_width(120) // fallback for ContentArrangement::Dynamic
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["Dependency", "Version", "MSRV", "Depends on"]);

                table
            },
            |acc, next| {
                acc.add_row(vec![
                    Cell::new(&next.name),
                    Cell::new(&next.version),
                    Cell::new(&next.msrv),
                    Cell::new(&next.dependencies.join(", ")),
                ]);
            },
        );

        table.fmt(f)
    }
}

impl std::fmt::Display for DirectDependenciesFormatter<crate::reporter::json::JsonPrinter<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Table of dependencies
        use json::object;

        let objects = self.direct_dependencies_msrv(Vec::new, |acc, next| {
            acc.push(object! {
                dependency: next.name,
                version: format!("{}", next.version),
                msrv: next.msrv,
                depends_on: next.dependencies,
            });
        });

        let json = object! {
            reason: "list",
            variant: crate::config::list::DIRECT_DEPS,
            success: true,
            list: objects,
        };

        writeln!(f, "{}", json)
    }
}

struct Values<'s, 'v> {
    name: &'s str,
    version: &'v crate::semver::Version,
    msrv: String,
    dependencies: Vec<String>,
}
