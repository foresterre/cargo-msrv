use crate::event::types::list_result::metadata::{format_version, package_msrv};
use crate::formatting::table;
use cargo_metadata::Package;
use cargo_msrv_context::types::ORDERED_BY_MSRV;
use cargo_msrv_rust_tools::DependencyGraph;
use msrv::MSRV;
use petgraph::visit::Bfs;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Formatter;

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
        let rows = dependencies(self.graph).map(Values::into_row);

        f.write_fmt(format_args!("{}", table(Values::HEADER, rows)))
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

    let mut version_map: BTreeMap<Option<MSRV>, Vec<&Package>> = BTreeMap::new();

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
            dependencies: packages.iter().map(|p| p.name.to_string()).collect(),
        })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct Values {
    msrv: Option<String>,
    dependencies: Vec<String>,
}

impl Values {
    const HEADER: [&'static str; 2] = ["MSRV", "Dependency"];

    fn into_row(self) -> [String; 2] {
        [self.msrv.unwrap_or_default(), self.dependencies.join(", ")]
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct SerializableValues {
    variant: &'static str,
    list: Vec<Values>,
}
