pub trait FormatUserOutput {
    /// Format user output as human-readable string
    fn format_human(&self) -> String;

    /// Format user output as json string
    fn format_json(&self) -> String;
}
