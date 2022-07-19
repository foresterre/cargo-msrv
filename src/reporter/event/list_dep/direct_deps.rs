use super::metadata::*;
use crate::config::list::DIRECT_DEPS;
use crate::dependency_graph::DependencyGraph;
use crate::formatting::table;
use tabled::{Style, Tabled};

pub struct DirectDepsFormatter<'g> {
    graph: &'g DependencyGraph,
}

impl<'g> DirectDepsFormatter<'g> {
    pub fn new(graph: &'g DependencyGraph) -> Self {
        Self { graph }
    }
}

impl ToString for DirectDepsFormatter<'_> {
    fn to_string(&self) -> String {
        let values = dependencies(self.graph);

        table(values).with(Style::modern()).to_string()
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

        let msrv = super::metadata::package_msrv(package);

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

#[derive(Debug, serde::Serialize)]
struct Values<'a> {
    name: &'a str,
    version: &'a crate::semver::Version,
    msrv: String,
    dependencies: Vec<String>,
}

impl Tabled for Values<'_> {
    const LENGTH: usize = 4;

    fn fields(&self) -> Vec<String> {
        vec![
            self.name.to_string(),
            self.version.to_string(),
            self.msrv.to_string(),
            self.dependencies.join(", "),
        ]
    }

    fn headers() -> Vec<String> {
        vec![
            "Name".to_string(),
            "Version".to_string(),
            "MSRV".to_string(),
            "Depends on".to_string(),
        ]
    }
}

#[derive(serde::Serialize)]
struct SerializableValues<'v> {
    variant: &'static str,
    list: Vec<Values<'v>>,
}
