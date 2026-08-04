use std::str::FromStr;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum TracingTargetOption {
    #[default]
    File,
    Stdout,
}

impl TracingTargetOption {
    pub const FILE: &'static str = "file";
    pub const STDOUT: &'static str = "stdout";
}

impl FromStr for TracingTargetOption {
    type Err = ParseTracingTargetOptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::FILE => Ok(Self::File),
            Self::STDOUT => Ok(Self::Stdout),
            unknown => Err(ParseTracingTargetOptionError(unknown.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Given log target '{0}' is not valid")]
pub struct ParseTracingTargetOptionError(pub String);
