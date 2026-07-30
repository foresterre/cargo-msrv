use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Progress bar rendered to stderr
    #[default]
    Human,
    /// Json status updates printed to stdout
    Json,
    /// Minimal output, usually just the result, such as the MSRV or whether verify succeeded or failed
    Minimal,
    /// No output -- meant to be used for debugging and testing
    #[value(skip)]
    None,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Json => write!(f, "json"),
            Self::Minimal => write!(f, "minimal"),
            Self::None => write!(f, "none"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = ParseOutputFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            "minimal" => Ok(Self::Minimal),
            unknown => Err(ParseOutputFormatError(unknown.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Given output format '{0}' is not valid")]
pub struct ParseOutputFormatError(pub String);
