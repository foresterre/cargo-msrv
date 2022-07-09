use crate::dependency_graph::DependencyGraph;
use crate::Event;
use crate::Message;

use direct_deps::DirectDepsFormatter;
use ordered_by_msrv::OrderedByMsrvFormatter;

mod direct_deps;
mod metadata;
mod ordered_by_msrv;

#[derive(Clone, Debug, PartialEq)]
pub struct ListDep {
    variant: ListMsrvVariant,
    graph: DependencyGraph,
}

impl ListDep {
    pub fn new(variant: ListMsrvVariant, graph: DependencyGraph) -> Self {
        Self { variant, graph }
    }
}

impl From<ListDep> for Event {
    fn from(it: ListDep) -> Self {
        Message::ListDep(it).into()
    }
}

impl ToString for ListDep {
    fn to_string(&self) -> String {
        match self.variant {
            ListMsrvVariant::DirectDeps => DirectDepsFormatter::new(&self.graph).to_string(),
            ListMsrvVariant::OrderedByMSRV => OrderedByMsrvFormatter::new(&self.graph).to_string(),
        }
    }
}

impl serde::Serialize for ListDep {
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
