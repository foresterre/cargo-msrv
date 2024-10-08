use super::display_option;
use super::display_vec;
use super::metadata::*;
use crate::context::list::DIRECT_DEPS;
use crate::dependency_graph::DependencyGraph;
use crate::reporter::formatting::table;
use std::fmt;
use std::fmt::Formatter;
use tabled::{Style, Tabled};

pub struct DirectDepsFormatter<'g> {
    graph: &'g DependencyGraph,
}

impl<'g> DirectDepsFormatter<'g> {
    pub fn new(graph: &'g DependencyGraph) -> Self {
        Self { graph }
    }
}

impl fmt::Display for DirectDepsFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let values = dependencies(self.graph);

        f.write_fmt(format_args!("{}", table(values).with(Style::modern())))
    }
}

impl serde::Serialize for DirectDepsFormatter<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable = SerializableValues {
            variant: DIRECT_DEPS,
            list: dependencies(self.graph).collect(),
        };

        serializable.serialize(serializer)
    }
}

fn dependencies(graph: &DependencyGraph) -> impl Iterator<Item = Values> {
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

#[derive(Debug, serde::Serialize, Tabled)]
struct Values<'a> {
    #[tabled(rename = "Name")]
    name: &'a str,
    #[tabled(rename = "Version")]
    version: &'a crate::semver::Version,
    #[tabled(rename = "MSRV", display_with = "display_option")]
    msrv: Option<String>,
    #[tabled(rename = "Depends on", display_with = "display_vec")]
    dependencies: Vec<String>,
}

#[derive(serde::Serialize)]
struct SerializableValues<'v> {
    variant: &'static str,
    list: Vec<Values<'v>>,
}
