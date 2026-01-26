//! Error types for SCID database parsing
//!
//! This module defines all error types that can occur during SCID file parsing
//! and PGN conversion. All errors use the `thiserror` crate for ergonomic
//! error handling.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for SCID operations
///
/// All fallible operations in this library return this Result type.
pub type Result<T> = std::result::Result<T, ScidError>;

/// Errors that can occur during SCID database parsing and conversion
#[derive(Debug, Error)]
pub enum ScidError {
    /// I/O error occurred while reading files
    ///
    /// This wraps standard library I/O errors that occur when reading
    /// SCID database files from disk.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid SCID file format detected
    ///
    /// This error occurs when a file doesn't match the expected SCID format,
    /// such as having an incorrect magic number or version.
    ///
    /// # Examples
    ///
    /// - Wrong magic bytes (not "Scid.si\0" or "Scid.sn\0")
    /// - Unsupported SCID version number
    /// - Truncated file header
    #[error("Invalid SCID file: {0}")]
    InvalidFormat(String),

    /// Parse error at specific location in file
    ///
    /// This error provides detailed context about where parsing failed,
    /// including the file being parsed, byte offset, and description.
    ///
    /// This is the most common error type and includes rich context for debugging.
    #[error("Parse error in {file:?} at offset {offset}: {message}")]
    ParseError {
        /// Path to the file being parsed
        file: PathBuf,
        /// Byte offset where the error occurred
        offset: u64,
        /// Description of what went wrong
        message: String,
    },

    /// Invalid game index requested
    ///
    /// Attempted to access a game that doesn't exist in the database.
    ///
    /// # Examples
    ///
    /// - Requesting game 100 when database only has 50 games
    /// - Negative game index (if using signed integers)
    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),

    /// Character encoding error
    ///
    /// Occurs when name strings or comments contain invalid UTF-8 sequences.
    /// SCID files should use UTF-8, but older databases may have encoding issues.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Data validation error
    ///
    /// The file structure is valid but the data doesn't make sense.
    ///
    /// # Examples
    ///
    /// - Date with month > 12
    /// - ELO rating > 4095 (exceeds 12-bit limit)
    /// - Checksum mismatch
    #[error("Validation error: {0}")]
    Validation(String),

    /// Unsupported feature or format variation
    ///
    /// The file uses a SCID feature that isn't yet implemented.
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

impl ScidError {
    /// Create a parse error with context
    ///
    /// Convenience constructor for the most common error type.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::error::ScidError;
    /// use std::path::PathBuf;
    ///
    /// let error = ScidError::parse(
    ///     PathBuf::from("test.si4"),
    ///     182,
    ///     "Invalid game count"
    /// );
    /// ```
    pub fn parse(file: PathBuf, offset: u64, message: impl Into<String>) -> Self {
        ScidError::ParseError {
            file,
            offset,
            message: message.into(),
        }
    }

    /// Create an invalid format error
    ///
    /// Convenience constructor for format validation errors.
    pub fn invalid_format(message: impl Into<String>) -> Self {
        ScidError::InvalidFormat(message.into())
    }

    /// Create a validation error
    ///
    /// Convenience constructor for data validation errors.
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
