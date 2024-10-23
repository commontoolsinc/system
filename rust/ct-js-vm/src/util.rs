/// Converts an error to a string.
pub fn format_error<E: std::fmt::Display>(error: E) -> String {
    format!("{}", error)
}
