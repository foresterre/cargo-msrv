use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::LogLevel;

pub const VALUES: CliValues<LogLevel> = CliValues::new(&[
    CliValue::new("trace", LogLevel::Trace),
    CliValue::new("debug", LogLevel::Debug),
    CliValue::new("info", LogLevel::Info),
    CliValue::new("warn", LogLevel::Warn),
    CliValue::new("error", LogLevel::Error),
]);
