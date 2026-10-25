use super::highlight::PackageIdSpecQuery;
use super::metadata::*;
use super::render_package_name;
use crate::formatting::table;
use cargo_msrv_context::types::DIRECT_DEPS;
use cargo_msrv_rust_tools::DependencyGraph;
use cargo_util_schemas::core::PackageIdSpec;
use std::fmt;
use std::fmt::Formatter;

pub struct DirectDepsFormatter<'g> {
    graph: &'g DependencyGraph,
    highlight: &'g [PackageIdSpec],
}

impl<'g> DirectDepsFormatter<'g> {
    pub fn new(graph: &'g DependencyGraph, highlight: &'g [PackageIdSpec]) -> Self {
        Self { graph, highlight }
    }
}

impl fmt::Display for DirectDepsFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let rows = dependencies(self.graph, self.highlight).map(Values::into_row);
        f.write_fmt(format_args!("{}", table(Values::HEADER, rows)))
    }
}

impl serde::Serialize for DirectDepsFormatter<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable = SerializableValues {
            variant: DIRECT_DEPS,
            list: dependencies(self.graph, self.highlight).collect(),
        };

        serializable.serialize(serializer)
    }
}

fn dependencies<'g>(
    graph: &'g DependencyGraph,
    highlight: &'g [PackageIdSpec],
) -> impl Iterator<Item = Values<'g>> {
    let package_id = graph.root_crate();
    let root_index = graph.index()[package_id].into();
    let neighbors = graph
        .packages()
        .neighbors_directed(root_index, petgraph::Direction::Outgoing);

    neighbors.map(move |dependency| {
        let package = &graph.packages()[dependency];
        let msrv = package_msrv(package);

        Values {
            name: &package.name,
            highlighted: highlight.iter().any(|spec| spec.matches(package)),
            version: &package.version,
            msrv: format_version(msrv.as_ref()),
            dependencies: package
                .dependencies
                .iter()
                .map(|d| d.name.clone())
                .collect(),
        }
    })
}

#[derive(Debug, serde::Serialize)]
struct Values<'a> {
    name: &'a str,
    #[serde(skip)]
    highlighted: bool,
    version: &'a semver::Version,
    msrv: Option<String>,
    dependencies: Vec<String>,
}

impl Values<'_> {
    const HEADER: [&'static str; 4] = ["Name", "Version", "MSRV", "Depends on"];

    fn into_row(self) -> [String; 4] {
        [
            render_package_name(self.name, self.highlighted),
            self.version.to_string(),
            self.msrv.unwrap_or_default(),
            self.dependencies.join(", "),
        ]
    }
}

#[derive(serde::Serialize)]
struct SerializableValues<'v> {
    variant: &'static str,
    list: Vec<Values<'v>>,
}
