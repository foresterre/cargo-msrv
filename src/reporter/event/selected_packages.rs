use crate::reporter::{Event, Message};
use camino::Utf8PathBuf;

/// Workspace packages selected
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectedPackages {
    package_names: Option<Vec<SelectedPackage>>,
}

impl SelectedPackages {
    pub fn new(package_names: Option<Vec<SelectedPackage>>) -> Self {
        Self { package_names }
    }
}

impl From<SelectedPackages> for Event {
    fn from(it: SelectedPackages) -> Self {
        Message::SelectedPackages(it).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectedPackage {
    pub name: String,
    pub path: Utf8PathBuf,
}
