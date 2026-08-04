use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::TracingTargetOption;

pub const VALUES: CliValues<TracingTargetOption> = CliValues::new(&[
    CliValue::new(TracingTargetOption::FILE, TracingTargetOption::File),
    CliValue::new(TracingTargetOption::STDOUT, TracingTargetOption::Stdout),
]);
