use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ScidError>;

#[derive(Debug, Error)]
pub enum ScidError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Invalid date format")]
    InvalidDate,
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid file format: {message}")]
    InvalidFormat { message: String },
    #[error("Parse error at offset {offset}: {message}")]
    ParseError { offset: usize, message: String },
    #[error("Chess error: {0}")]
    Chess(String),
    #[error("File open error: {0} on file {1:?}")]
    FileOpen(std::io::Error, PathBuf),
    #[error("Mmap error: {0} on file {1:?}")]
    Mmap(std::io::Error, PathBuf),
    #[error("Conversion error: {message}")]
    ConversionError { message: String },
    #[error("Name lookup error: {message}")]
    NameLookupError { message: String },
    #[error("Game index {index} out of bounds (max: {max})")]
    GameIndexOutOfBounds { index: usize, max: usize },
    #[error("Invalid game data: {message}")]
    InvalidGameData { message: String },
    #[error("Missing required data: {message}")]
    MissingData { message: String },
}

impl ScidError {
    pub fn conversion_error(message: impl Into<String>) -> Self {
        ScidError::ConversionError {
            message: message.into(),
        }
    }

    pub fn invalid_format(message: impl Into<String>) -> Self {
        ScidError::InvalidFormat {
            message: message.into(),
        }
    }

    pub fn parse_error(offset: usize, message: impl Into<String>) -> Self {
        ScidError::ParseError {
            offset,
            message: message.into(),
        }
    }

    pub fn name_lookup_error(message: impl Into<String>) -> Self {
        ScidError::NameLookupError {
            message: message.into(),
        }
    }

    pub fn game_index_out_of_bounds(index: usize, max: usize) -> Self {
        ScidError::GameIndexOutOfBounds { index, max }
    }

    pub fn invalid_game_data(message: impl Into<String>) -> Self {
        ScidError::InvalidGameData {
            message: message.into(),
        }
    }

    pub fn missing_data(message: impl Into<String>) -> Self {
        ScidError::MissingData {
            message: message.into(),
        }
    }
}

pub trait ResultExt<T> {
    fn invalid_format(self, message: impl Into<String>) -> Result<T>;
    fn parse_error(self, offset: usize, message: impl Into<String>) -> Result<T>;
    fn conversion_error(self, message: impl Into<String>) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E> {
    fn invalid_format(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|_| ScidError::invalid_format(message))
    }

    fn parse_error(self, offset: usize, message: impl Into<String>) -> Result<T> {
        self.map_err(|_| ScidError::parse_error(offset, message))
    }

    fn conversion_error(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|_| ScidError::conversion_error(message))
    }
}

pub trait OptionExt<T> {
    fn missing_data(self, message: impl Into<String>) -> Result<T>;
    fn invalid_format(self, message: impl Into<String>) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn missing_data(self, message: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| ScidError::missing_data(message))
    }

    fn invalid_format(self, message: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| ScidError::invalid_format(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ScidError::invalid_format("test message");
        assert!(matches!(err, ScidError::InvalidFormat { .. }));

        let err = ScidError::parse_error(100, "test parse");
        assert!(matches!(err, ScidError::ParseError { .. }));

        let err = ScidError::conversion_error("conversion failed".to_string());
        assert!(matches!(err, ScidError::ConversionError { .. }));
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<i32, &str> = Err("original error");
        let converted = result.invalid_format("format error");
        assert!(matches!(converted, Err(ScidError::InvalidFormat { .. })));
    }

    #[test]
    fn test_option_ext() {
        let option: Option<i32> = None;
        let result = option.missing_data("data missing");
        assert!(matches!(result, Err(ScidError::MissingData { .. })));
    }
}
