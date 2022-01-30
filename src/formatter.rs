pub trait FormatUserOutput<T> {
    /// Format user output as output type `T`.
    fn format_line(&self) -> String;
}

pub struct Json;
pub struct Human;
