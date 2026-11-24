use crate::Event;
use crate::event::Message;
use cargo_msrv_context::SelectedPackage;
use cargo_msrv_rust_tools::{DependencyGraph, PackageGraphIndex};
use cargo_util_schemas::core::PackageIdSpec;
use owo_colors::OwoColorize;
use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

use crate::event::subcommand_result::SubcommandResult;
use crate::event::types::list_result::ordered_by_msrv::OrderedByMsrvFormatter;
use cargo_msrv_context::types::ListMsrvVariant;
use direct_deps::DirectDepsFormatter;

mod direct_deps;
mod highlight;
mod metadata;
mod ordered_by_msrv;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<SelectedPackage>,
    result: ResultDetails,
}

impl ListResult {
    pub fn new(
        variant: ListMsrvVariant,
        graph: Arc<DependencyGraph>,
        root: PackageGraphIndex,
        highlight: Vec<PackageIdSpec>,
    ) -> Self {
        Self {
            package: None,
            result: ResultDetails {
                variant,
                graph,
                root,
                highlight,
            },
        }
    }

    pub fn with_package(mut self, package: Option<SelectedPackage>) -> Self {
        self.package = package;
        self
    }

    pub fn package(&self) -> Option<&SelectedPackage> {
        self.package.as_ref()
    }
}

impl fmt::Display for ListResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.result))
    }
}

impl From<ListResult> for SubcommandResult {
    fn from(it: ListResult) -> Self {
        SubcommandResult::List(it)
    }
}

impl From<ListResult> for Event {
    fn from(it: ListResult) -> Self {
        Message::SubcommandResult(it.into()).into()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResultDetails {
    variant: ListMsrvVariant,
    graph: Arc<DependencyGraph>,
    root: PackageGraphIndex,
    highlight: Vec<PackageIdSpec>,
}

impl fmt::Display for ResultDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.variant {
            ListMsrvVariant::DirectDeps => f.write_fmt(format_args!(
                "{}",
                DirectDepsFormatter::new(&self.graph, self.root, &self.highlight)
            )),
            ListMsrvVariant::OrderedByMSRV => f.write_fmt(format_args!(
                "{}",
                OrderedByMsrvFormatter::new(&self.graph, self.root, &self.highlight)
            )),
        }
    }
}

impl serde::Serialize for ResultDetails {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.variant {
            ListMsrvVariant::DirectDeps => {
                DirectDepsFormatter::new(&self.graph, self.root, &self.highlight)
                    .serialize(serializer)
            }
            ListMsrvVariant::OrderedByMSRV => {
                OrderedByMsrvFormatter::new(&self.graph, self.root, &self.highlight)
                    .serialize(serializer)
            }
        }
    }
}

fn render_package_name(name: &str, highlighted: bool) -> String {
    if highlighted {
        name.reversed().to_string()
    } else {
        name.to_string()
    }
}
