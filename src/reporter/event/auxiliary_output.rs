use crate::reporter::event::Message;
use crate::Event;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
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

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Destination {
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Item {
    MSRV { kind: MSRVKind },
    ToolchainFile { kind: ToolchainFileKind },
}

impl Item {
    pub fn msrv(kind: MSRVKind) -> Self {
        Self::MSRV { kind }
    }

    pub fn toolchain_file(kind: ToolchainFileKind) -> Self {
        Self::ToolchainFile { kind }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MSRVKind {
    RustVersion,
    MetadataFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolchainFileKind {
    /* Legacy, : Unsupported right now */
    Toml,
}
