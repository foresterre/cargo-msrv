use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::OutputFormat;

// NB: `OutputFormat::None` is intentionally not selectable: it is enabled by `--no-user-output`.
pub const VALUES: CliValues<OutputFormat> = CliValues::new(&[
    CliValue::new("human", OutputFormat::Human).help("Progress bar rendered to stderr"),
    CliValue::new("json", OutputFormat::Json).help("Json status updates printed to stdout"),
    CliValue::new("minimal", OutputFormat::Minimal).help(
        "Minimal output, usually just the result, such as the MSRV or whether verify succeeded or failed",
    ),
]);
