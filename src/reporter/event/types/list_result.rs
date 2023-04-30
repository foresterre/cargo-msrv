use crate::dependency_graph::DependencyGraph;
use crate::reporter::event::Message;
use crate::Event;
use std::borrow::Cow;

use crate::context::list::ListMsrvVariant;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::types::list_result::ordered_by_msrv::OrderedByMsrvFormatter;
use direct_deps::DirectDepsFormatter;

mod direct_deps;
mod metadata;
mod ordered_by_msrv;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListResult {
    result: ResultDetails,
}

impl ListResult {
    pub fn new(variant: ListMsrvVariant, graph: DependencyGraph) -> Self {
        Self {
            result: ResultDetails { variant, graph },
        }
    }
}

impl ToString for ListResult {
    fn to_string(&self) -> String {
        self.result.to_string()
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
    graph: DependencyGraph,
}

impl ToString for ResultDetails {
    fn to_string(&self) -> String {
        match self.variant {
            ListMsrvVariant::DirectDeps => DirectDepsFormatter::new(&self.graph).to_string(),
            ListMsrvVariant::OrderedByMSRV => OrderedByMsrvFormatter::new(&self.graph).to_string(),
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
                DirectDepsFormatter::new(&self.graph).serialize(serializer)
            }
            ListMsrvVariant::OrderedByMSRV => {
                OrderedByMsrvFormatter::new(&self.graph).serialize(serializer)
            }
        }
    }
}

fn display_option(option: &Option<String>) -> Cow<'static, str> {
    match option {
        Some(s) => Cow::from(s.to_string()),
        None => Cow::from(""),
    }
}

fn display_vec(vec: &[String]) -> Cow<'static, str> {
    Cow::from(vec.join(", "))
}
