use crate::reporter::event::Message;
use crate::Event;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuxiliaryOutput {
    destination: Destination,
    item: Item,
}

impl AuxiliaryOutput {
    pub fn new(destination: Destination, item: Item) -> Self {
        Self { destination, item }
    }
}

impl From<AuxiliaryOutput> for Event {
    fn from(it: AuxiliaryOutput) -> Self {
        Message::AuxiliaryOutput(it).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Destination {
    File { path: PathBuf },
}

impl Destination {
    pub fn file(path: PathBuf) -> Self {
        Self::File { path }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Item {
    Msrv { kind: MsrvKind },
    ToolchainFile { kind: ToolchainFileKind },
}

impl Item {
    pub fn msrv(kind: MsrvKind) -> Self {
        Self::Msrv { kind }
    }

    pub fn toolchain_file(kind: ToolchainFileKind) -> Self {
        Self::ToolchainFile { kind }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MsrvKind {
    // The package.rust-version as supported by the Cargo Manifest format.
    RustVersion,
    // The package.metadata.msrv key used as fallback for crates where the Cargo Manifest format did
    // not support the package.rust-version key yet.
    MetadataFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolchainFileKind {
    /* Legacy, : Unsupported right now */
    Toml,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use crate::Event;
    use std::path::Path;
    use storyteller::Reporter;

    #[yare::parameterized(
        rust_version_msrv = { Item::msrv(MsrvKind::RustVersion) },
        metadata_fallback_msrv = { Item::msrv(MsrvKind::MetadataFallback) },
        toolchain_file_toml = { Item::toolchain_file(ToolchainFileKind::Toml) },
    )]
    fn reported_action(item: Item) {
        let reporter = TestReporterWrapper::default();
        let event = AuxiliaryOutput::new(Destination::file(Path::new("hello").to_path_buf()), item);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::AuxiliaryOutput(event)),]
        );
    }
}
