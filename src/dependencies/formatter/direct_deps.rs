use crate::config::OutputFormat;
use crate::dependencies::formatter::format_version;
use crate::dependencies::DependencyGraph;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};

pub(crate) fn format(graph: &DependencyGraph, format: OutputFormat) -> Option<String> {
    match format {
        OutputFormat::Human => {
            let values = dependencies(graph);
            Some(format_human(values))
        }
        OutputFormat::Json => {
            let values = dependencies(graph);
            Some(format_json(values))
        }
        OutputFormat::None | OutputFormat::TestSuccesses => None,
    }
}

struct Values<'a> {
    name: &'a str,
    version: &'a crate::semver::Version,
    msrv: String,
    dependencies: Vec<String>,
}

fn dependencies(graph: &DependencyGraph) -> impl Iterator<Item = Values> {
    let package_id = &graph.root_crate;
    let root_index = graph.index[package_id].into();
    let neighbors = graph
        .packages
        .neighbors_directed(root_index, petgraph::Direction::Outgoing);

    neighbors.map(move |dependency| {
        let package = &graph.packages[dependency];

        let msrv = super::msrv(package);

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

fn format_human<'a>(values: impl Iterator<Item = Values<'a>>) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_table_width(120) // fallback for ContentArrangement::Dynamic
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Dependency", "Version", "MSRV", "Depends on"]);

    for value in values {
        table.add_row(vec![
            Cell::new(&value.name),
            Cell::new(&value.version),
            Cell::new(&value.msrv),
            Cell::new(&value.dependencies.join(", ")),
        ]);
    }

    table.to_string()
}

fn format_json<'a>(values: impl Iterator<Item = Values<'a>>) -> String {
    let objects: Vec<_> = values
        .map(|value| {
            json::object! {
                dependency: value.name,
                version: value.version.to_string(),
                msrv: value.msrv,
                depends_on: value.dependencies,
            }
        })
        .collect();

    let json = json::object! {
        reason: "list",
        variant: crate::config::list::DIRECT_DEPS,
        success: true,
        list: objects,
    };

    json.to_string()
}
