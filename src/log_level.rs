use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn variants() -> &'static [&'static str] {
        &["trace", "debug", "info", "warn", "error"]
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn parse_num_level(input: &str) -> Option<LogLevel> {
            input.parse::<u8>().ok().and_then(|number| match number {
                1 => Some(LogLevel::Error),
                2 => Some(LogLevel::Warn),
                3 => Some(LogLevel::Info),
                4 => Some(LogLevel::Debug),
                5 => Some(LogLevel::Trace),
                _ => None,
            })
        }

        fn parse_str_level(input: &str) -> Option<LogLevel> {
            match input {
                s if s.eq_ignore_ascii_case("error") => Some(LogLevel::Error),
                s if s.eq_ignore_ascii_case("warn") => Some(LogLevel::Warn),
                s if s.eq_ignore_ascii_case("info") => Some(LogLevel::Info),
                s if s.eq_ignore_ascii_case("debug") => Some(LogLevel::Debug),
                s if s.eq_ignore_ascii_case("trace") => Some(LogLevel::Trace),
                _ => None,
            }
        }

        parse_num_level(input)
            .or_else(|| parse_str_level(input))
            .ok_or_else(|| ParseLogLevelError::NoMatchingLevel {
                given_input: input.to_string(),
                valid_options_formatted: Self::variants().join(","),
            })
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseLogLevelError {
    #[error("The given log level '{given_input}' does not exist, valid options are: {valid_options_formatted}]")]
    NoMatchingLevel {
        given_input: String,
        valid_options_formatted: String,
    },
}

#[cfg(test)]
mod tests {
    use crate::log_level::{LogLevel, ParseLogLevelError};

    #[yare::parameterized(
        trace_numeric = {  "5", LogLevel::Trace },
        trace_alphabetic = { "trace", LogLevel::Trace },
        debug_numeric = {  "4", LogLevel::Debug },
        debug_alphabetic = { "debug", LogLevel::Debug },
        debug_alphabetic_uppercase = { "DEBUG", LogLevel::Debug },
        info_numeric = {  "3", LogLevel::Info },
        info_alphabetic = { "info", LogLevel::Info },
        info_alphabetic_uppercase = { "INFO", LogLevel::Info },
        warn_numeric = {  "2", LogLevel::Warn },
        warn_alphabetic = { "warn", LogLevel::Warn },
        warn_alphabetic_uppercase = { "WARN", LogLevel::Warn },
        error_numeric = {  "1", LogLevel::Error },
        error_alphabetic = { "error", LogLevel::Error },
        error_alphabetic_uppercase = { "ERROR", LogLevel::Error },
    )]
    fn valid_log_levels(input: &str, expected: LogLevel) {
        assert_eq!(input.parse::<LogLevel>().unwrap(), expected);
    }

    #[yare::parameterized(
        numeric_floor = { "0" },
        numeric_ceil = { "6" },
        non_existing = { "null" },
        empty = { "" },
    )]
    fn invalid_log_levels(input: &str) {
        let expected_err = input.parse::<LogLevel>().unwrap_err();

        match expected_err {
            ParseLogLevelError::NoMatchingLevel {
                ref given_input, ..
            } => assert_eq!(given_input, input),
        };
    }
}
