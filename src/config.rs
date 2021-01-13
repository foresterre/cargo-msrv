use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CmdMatches<'a> {
    target: String,
    check_command: Vec<&'a str>,
    seek_path: Option<PathBuf>,
}

impl<'a> CmdMatches<'a> {
    pub fn new(target: String) -> Self {
        Self {
            target,
            check_command: vec!["cargo", "build", "--all"],
            seek_path: None,
        }
    }

    pub fn target(&self) -> &String {
        &self.target
    }

    pub fn custom_check(&self) -> &Vec<&'a str> {
        &self.check_command
    }

    pub fn seek_path(&self) -> Option<&Path> {
        self.seek_path.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct CmdMatchesBuilder<'a> {
    inner: CmdMatches<'a>,
}

impl<'a> CmdMatchesBuilder<'a> {
    pub fn new(default_target: &str) -> Self {
        Self {
            inner: CmdMatches::new(default_target.to_string()),
        }
    }

    pub fn target(mut self, target: &str) -> Self {
        self.inner.target = target.to_string();
        self
    }

    pub fn check_command(mut self, cmd: Vec<&'a str>) -> Self {
        self.inner.check_command = cmd;
        self
    }

    pub fn seek_path<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.inner.seek_path = path.map(|p| PathBuf::from(p.as_ref()));
        self
    }

    pub fn build(self) -> CmdMatches<'a> {
        self.inner
    }
}
