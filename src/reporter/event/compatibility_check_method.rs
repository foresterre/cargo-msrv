use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CompatibilityCheckMethod {
    toolchain: OwnedToolchainSpec,
    method: Method,
}

impl CompatibilityCheckMethod {
    pub fn new(toolchain: impl Into<OwnedToolchainSpec>, method: Method) -> Self {
        Self {
            toolchain: toolchain.into(),
            method,
        }
    }
}

impl IntoIdentifiableEvent for CompatibilityCheckMethod {
    fn identifier(&self) -> &'static str {
        "compatibility_check_method"
    }
}

impl From<CompatibilityCheckMethod> for Event {
    fn from(it: CompatibilityCheckMethod) -> Self {
        Message::CompatibilityCheckMethod(it).into()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    RustupRun {
        args: Vec<String>,
        path: Option<PathBuf>,
    },
}

impl Method {
    pub fn rustup_run(
        args: impl IntoIterator<Item = impl AsRef<str>>,
        path: Option<impl AsRef<Path>>,
    ) -> Self {
        Self::RustupRun {
            args: args.into_iter().map(|s| s.as_ref().to_string()).collect(),
            path: path.as_ref().map(|path| path.as_ref().to_path_buf()),
        }
    }
}
