pub struct CliError;

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self
    }
}
