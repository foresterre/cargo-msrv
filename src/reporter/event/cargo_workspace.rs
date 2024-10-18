use crate::reporter::{Event, Message};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CargoWorkspace {
    package_names: Vec<String>,
}

impl CargoWorkspace {
    pub fn new(package_names: Vec<String>) -> Self {
        Self { package_names }
    }
}

impl From<CargoWorkspace> for Event {
    fn from(it: CargoWorkspace) -> Self {
        Message::CargoWorkspace(it).into()
    }
}
