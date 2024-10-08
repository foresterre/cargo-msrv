use super::display_option;
use super::display_vec;
use crate::context::list::ORDERED_BY_MSRV;
use crate::dependency_graph::DependencyGraph;
use crate::reporter::event::types::list_result::metadata::{format_version, package_msrv};
use crate::reporter::formatting::table;
use crate::semver;
use cargo_metadata::Package;
use petgraph::visit::Bfs;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Formatter;
use tabled::{Style, Tabled};

pub struct OrderedByMsrvFormatter<'g> {
    graph: &'g DependencyGraph,
}

impl<'g> OrderedByMsrvFormatter<'g> {
    pub fn new(graph: &'g DependencyGraph) -> Self {
        Self { graph }
    }
}

impl fmt::Display for OrderedByMsrvFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let values = dependencies(self.graph);

        f.write_fmt(format_args!("{}", table(values).with(Style::modern())))
    }
}

impl serde::Serialize for OrderedByMsrvFormatter<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable = SerializableValues {
            variant: ORDERED_BY_MSRV,
            list: dependencies(self.graph).collect(),
        };

        serializable.serialize(serializer)
    }
}

fn dependencies(graph: &DependencyGraph) -> impl Iterator<Item = Values> + '_ {
    let package_id = &graph.root_crate();
    let root_index = graph.index()[package_id].into();
    let mut bfs = Bfs::new(graph.packages(), root_index);

    let mut version_map: BTreeMap<Option<semver::Version>, Vec<&Package>> = BTreeMap::new();

    while let Some(nx) = bfs.next(graph.packages()) {
        let package = &graph.packages()[nx];

        let msrv = package_msrv(package);

        version_map.entry(msrv).or_default().push(package);
    }

    version_map
        .into_iter()
        .rev()
        .map(|(version, packages)| Values {
            msrv: format_version(version.as_ref()),
            dependencies: packages.iter().map(|p| p.name.clone()).collect(),
        })
}

#[derive(Debug, serde::Serialize, Tabled)]
#[serde(rename_all = "snake_case")]
struct Values {
    #[tabled(rename = "MSRV", display_with = "display_option")]
    msrv: Option<String>,
    #[tabled(rename = "Dependency", display_with = "display_vec")]
    dependencies: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct SerializableValues {
    variant: &'static str,
    list: Vec<Values>,
}
