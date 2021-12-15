use crate::config::OutputFormat;
use crate::dependencies::formatter::{
    format_version, get_package_metadata_msrv, parse_manifest_workaround,
};
use crate::dependencies::DependencyGraph;
use crate::semver;
use cargo_metadata::Package;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};
use petgraph::visit::Bfs;
use std::collections::BTreeMap;

pub(crate) fn format(format: OutputFormat, graph: &DependencyGraph) -> Option<String> {
    match format {
        OutputFormat::Human => {
            let values = dependencies_by_msrv(graph);
            Some(format_human(values))
        }
        OutputFormat::Json => {
            let values = dependencies_by_msrv(graph);
            Some(format_json(values))
        }
        OutputFormat::None | OutputFormat::TestSuccesses => None,
    }
}

struct Values {
    msrv: String,
    dependencies: Vec<String>,
}

fn dependencies_by_msrv(graph: &DependencyGraph) -> impl Iterator<Item = Values> + '_ {
    let package_id = &graph.root_crate;
    let root_index = graph.index[package_id].into();
    let mut bfs = Bfs::new(&graph.packages, root_index);

    let mut version_map: BTreeMap<Option<semver::Version>, Vec<&Package>> = BTreeMap::new();

    while let Some(nx) = bfs.next(&graph.packages) {
        let package = &graph.packages[nx];

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

        version_map.entry(msrv).or_default().push(package);
    }

    version_map.into_iter().map(|(version, packages)| Values {
        msrv: format_version(version.as_ref()),
        dependencies: packages.iter().map(|p| p.name.clone()).collect(),
    })
}

fn format_human(values: impl Iterator<Item = Values>) -> String {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["MSRV", "Dependency"]);

    for value in values {
        table.add_row(vec![
            Cell::new(&value.msrv),
            Cell::new(&value.dependencies.join(", ")),
        ]);
    }

    table.to_string()
}

fn format_json(values: impl Iterator<Item = Values>) -> String {
    let objects: Vec<_> = values
        .map(|value| {
            json::object! {
                "msrv": value.msrv,
                "dependencies": value.dependencies
            }
        })
        .collect();

    let json = json::object! {
        reason: "list",
        variant: crate::config::list::ORDERED_BY_MSRV,
        success: true,
        list: objects,
    };

    json.to_string()
}
