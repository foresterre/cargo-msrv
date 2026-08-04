//! The selectable values of the command line options.
//!
//! The types themselves are defined by the [`cargo-msrv-context`](cargo_msrv_context) crate, which
//! is the programming interface of the program, and therefore should not depend on `clap`.
//!
//! Use this iso clap's `ValueEnum` derive macro.

use clap::builder::{PossibleValue, PossibleValuesParser, TypedValueParser};

pub mod list_msrv_variant;
pub mod log_level;
pub mod output_format;
pub mod release_source;
pub mod tracing_target_option;

pub struct CliValue<T> {
    name: &'static str,
    value: T,
    help: Option<&'static str>,
}

impl<T> CliValue<T> {
    pub const fn new(name: &'static str, value: T) -> Self {
        Self {
            name,
            value,
            help: None,
        }
    }

    pub const fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
}

pub struct CliValues<T: 'static> {
    values: &'static [CliValue<T>],
}

impl<T> CliValues<T> {
    pub const fn new(values: &'static [CliValue<T>]) -> Self {
        Self { values }
    }
}

impl<T> CliValues<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn parser(self) -> impl TypedValueParser<Value = T> {
        let values = self.values;

        PossibleValuesParser::new(values.iter().map(|value| {
            let possible_value = PossibleValue::new(value.name);

            match value.help {
                Some(help) => possible_value.help(help),
                None => possible_value,
            }
        }))
        .try_map(move |name| {
            values
                .iter()
                .find(|value| value.name == name)
                .map(|value| value.value.clone())
                .ok_or(UnknownValueError(name))
        })
    }

    pub fn default_value(self) -> &'static str
    where
        T: Default + PartialEq,
    {
        let default = T::default();

        self.values
            .iter()
            .find(|value| value.value == default)
            .map(|value| value.name)
            .expect("the default value must be selectable on the command line")
    }
}

impl<T> Clone for CliValues<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for CliValues<T> {}

#[derive(Debug, thiserror::Error)]
#[error("Given value '{0}' is not valid")]
pub struct UnknownValueError(String);

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_msrv_context::types::{
        ListMsrvVariant, LogLevel, OutputFormat, ReleaseSource, TracingTargetOption,
    };
    use std::ffi::OsStr;

    fn parse<T>(values: CliValues<T>, input: &str) -> Result<T, clap::Error>
    where
        T: Clone + Send + Sync + 'static,
    {
        values
            .parser()
            .parse_ref(&clap::Command::new("cargo-msrv"), None, OsStr::new(input))
    }

    #[yare::parameterized(
        human = { "human", OutputFormat::Human },
        json = { "json", OutputFormat::Json },
        minimal = { "minimal", OutputFormat::Minimal },
    )]
    fn parses_output_format(input: &str, expected: OutputFormat) {
        assert_eq!(parse(output_format::VALUES, input).unwrap(), expected);
    }

    #[yare::parameterized(
        trace = { "trace", LogLevel::Trace },
        debug = { "debug", LogLevel::Debug },
        info = { "info", LogLevel::Info },
        warn = { "warn", LogLevel::Warn },
        error = { "error", LogLevel::Error },
    )]
    fn parses_log_level(input: &str, expected: LogLevel) {
        assert_eq!(parse(log_level::VALUES, input).unwrap(), expected);
    }

    #[yare::parameterized(
        file = { "file", TracingTargetOption::File },
        stdout = { "stdout", TracingTargetOption::Stdout },
    )]
    fn parses_tracing_target_option(input: &str, expected: TracingTargetOption) {
        assert_eq!(
            parse(tracing_target_option::VALUES, input).unwrap(),
            expected
        );
    }

    #[yare::parameterized(
        direct_deps = { "direct-deps", ListMsrvVariant::DirectDeps },
        ordered_by_msrv = { "ordered-by-msrv", ListMsrvVariant::OrderedByMSRV },
    )]
    fn parses_list_msrv_variant(input: &str, expected: ListMsrvVariant) {
        assert_eq!(parse(list_msrv_variant::VALUES, input).unwrap(), expected);
    }

    #[test]
    fn parses_release_source() {
        assert_eq!(
            parse(release_source::VALUES, "rust-changelog").unwrap(),
            ReleaseSource::RustChangelog
        );

        #[cfg(feature = "rust-releases-dist-source")]
        assert_eq!(
            parse(release_source::VALUES, "rust-dist").unwrap(),
            ReleaseSource::RustDist
        );
    }

    #[test]
    fn defaults_are_selectable() {
        assert_eq!(output_format::VALUES.default_value(), "human");
        assert_eq!(log_level::VALUES.default_value(), "info");
        assert_eq!(tracing_target_option::VALUES.default_value(), "file");
        assert_eq!(list_msrv_variant::VALUES.default_value(), "ordered-by-msrv");
        assert_eq!(release_source::VALUES.default_value(), "rust-changelog");
    }

    #[yare::parameterized(
        no_user_output_is_not_selectable = { "none" },
        unknown = { "yaml" },
        empty = { "" },
    )]
    fn unknown_output_formats_are_rejected(input: &str) {
        assert!(parse(output_format::VALUES, input).is_err());
    }

    #[yare::parameterized(
        numeric = { "5" },
        uppercase = { "INFO" },
        unknown = { "verbose" },
    )]
    fn unknown_log_levels_are_rejected(input: &str) {
        assert!(parse(log_level::VALUES, input).is_err());
    }
}
