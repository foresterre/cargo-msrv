use crate::config::list::ORDERED_BY_MSRV;
use crate::dependency_graph::DependencyGraph;
use crate::formatting::table;
use crate::reporter::event::types::list_result::metadata::{format_version, package_msrv};
use crate::semver;
use cargo_metadata::Package;
use petgraph::visit::Bfs;
use std::collections::BTreeMap;
use tabled::{Style, Tabled};

pub struct OrderedByMsrvFormatter<'g> {
    graph: &'g DependencyGraph,
}

impl<'g> OrderedByMsrvFormatter<'g> {
    pub fn new(graph: &'g DependencyGraph) -> Self {
        Self { graph }
    }
}

impl ToString for OrderedByMsrvFormatter<'_> {
    fn to_string(&self) -> String {
        let values = dependencies(self.graph);

        table(values).with(Style::modern()).to_string()
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
    let mut bfs = Bfs::new(&graph.packages(), root_index);

    let mut version_map: BTreeMap<Option<semver::Version>, Vec<&Package>> = BTreeMap::new();

    while let Some(nx) = bfs.next(&graph.packages()) {
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

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct Values {
    msrv: String,
    dependencies: Vec<String>,
}

impl Tabled for Values {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<String> {
        let msrv = self.msrv.to_string();
        let deps = self.dependencies.join(", ");

        vec![msrv, deps]
    }

    fn headers() -> Vec<String> {
        vec!["MSRV".to_string(), "Dependency".to_string()]
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct SerializableValues {
    variant: &'static str,
    list: Vec<Values>,
}
