//! Error types for SCID database parsing

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for SCID operations
pub type Result<T> = std::result::Result<T, ScidError>;

/// Errors that can occur during SCID database parsing and conversion
#[derive(Debug, Error)]
pub enum ScidError {
    /// I/O error occurred while reading files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid SCID file format detected
    #[error("Invalid SCID file: {0}")]
    InvalidFormat(String),

    /// Parse error at specific location in file
    #[error("Parse error in {file:?} at offset {offset}: {message}")]
    ParseError {
        file: PathBuf,
        offset: u64,
        message: String,
    },

    /// Invalid game index requested
    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),

    /// Character encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Data validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Unsupported feature or format variation
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

impl ScidError {
    /// Create a parse error with context
    pub fn parse(file: PathBuf, offset: u64, message: impl Into<String>) -> Self {
        ScidError::ParseError {
            file,
            offset,
            message: message.into(),
        }
    }

    /// Create an invalid format error
    pub fn invalid_format(message: impl Into<String>) -> Self {
        ScidError::InvalidFormat(message.into())
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        ScidError::Validation(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ScidError::InvalidFormat("wrong magic".to_string());
        assert_eq!(err.to_string(), "Invalid SCID file: wrong magic");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ScidError::ParseError {
            file: PathBuf::from("test.si4"),
            offset: 100,
            message: "bad data".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("test.si4"));
        assert!(display.contains("100"));
        assert!(display.contains("bad data"));
    }

    #[test]
    fn test_parse_constructor() {
        let err = ScidError::parse(PathBuf::from("game.sg4"), 500, "invalid move");
        match err {
            ScidError::ParseError {
                file,
                offset,
                message,
            } => {
                assert_eq!(file, PathBuf::from("game.sg4"));
                assert_eq!(offset, 500);
                assert_eq!(message, "invalid move");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let scid_err: ScidError = io_err.into();
        assert!(matches!(scid_err, ScidError::Io(_)));
    }

    #[test]
    fn test_result_type_alias() {
        fn example_fn() -> Result<i32> {
            Ok(42)
        }
        assert_eq!(example_fn().unwrap(), 42);
    }
}
