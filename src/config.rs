use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CmdMatches {
    target: String,
    seek_cmd: String,
    seek_path: Option<PathBuf>,
}

impl CmdMatches {
    pub fn new(target: String) -> Self {
        Self {
            target,
            seek_cmd: "cargo build".to_string(),
            seek_path: None,
        }
    }

    pub fn target(&self) -> &String {
        &self.target
    }

    pub fn seek_cmd(&self) -> &String {
        &self.seek_cmd
    }

    pub fn seek_path(&self) -> Option<&Path> {
        self.seek_path.as_ref().map(|p| p.as_path())
    }
}

#[derive(Debug, Clone)]
pub struct CmdMatchesBuilder {
    inner: CmdMatches,
}

impl CmdMatchesBuilder {
    pub fn new(target: &str) -> Self {
        Self {
            inner: CmdMatches::new(target.to_string()),
        }
    }

    pub fn target(mut self, target: &str) -> Self {
        self.inner.target = target.to_string();
        self
    }

    pub fn seek_cmd(mut self, cmd: String) -> Self {
        self.inner.seek_cmd = cmd;
        self
    }

    pub fn seek_path<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.inner.seek_path = path.map(|p| PathBuf::from(p.as_ref()));
        self
    }

    pub fn build(self) -> CmdMatches {
        self.inner
    }
}
