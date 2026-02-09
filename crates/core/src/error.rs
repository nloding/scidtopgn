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
    /// Errors specific to game processing (potentially recoverable)
    #[error("Game processing error: {message}")]
    GameProcessError { message: String },

    /// Failed to decode move {move_num} (byte 0x{byte:02X}): {message}
    #[error("Failed to decode move {move_num} (byte 0x{byte:02X}): {message}")]
    MoveDecodeError {
        move_num: usize,
        byte: u8,
        message: String,
    },

    /// Invalid game structure: {message}
    #[error("Invalid game structure: {message}")]
    StructureError { message: String },

    /// I/O error occurred while reading files
    ///
    /// This wraps standard library I/O errors that occur when reading
    /// SCID database files from disk.
    ///
    /// # Examples
    ///
    /// - Wrong magic bytes (not "Scid.si\0" or "Scid.sn\0")
    /// - Unsupported SCID version number
    /// - Truncated file header
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

    /// Too many errors encountered during conversion
    ///
    /// This error is returned when the error count exceeds the limit
    /// specified in `ErrorMode::Lenient`.
    #[error("Too many errors: {count} encountered, max allowed: {max}")]
    TooManyErrors { count: usize, max: usize },

    /// Invalid game data encountered
    ///
    /// Game data structure is corrupted or malformed.
    #[error("Invalid game data: {0}")]
    InvalidGameData(String),

    /// Character encoding error
    ///
    /// Occurs when name strings or comments contain invalid UTF-8 sequences.
    /// SCID files should use UTF-8, but older databases may have encoding issues.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Decompression error
    ///
    /// Failed to decompress zlib-compressed game data.
    #[error("Decompression error: {0}")]
    DecompressionError(String),

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

    /// One or more database files not found
    ///
    /// All three required files (.si4, .sn4, .sg4) must exist.
    /// This error is returned when any file is missing.
    #[error("File not found: {file}")]
    FileNotFound { file: String },

    /// File cannot be opened
    ///
    /// The file exists but cannot be opened due to permissions,
    /// locked files, or other OS-level errors.
    #[error("Failed to open file {file}: {source}")]
    FileOpenError {
        file: String,
        source: std::io::Error,
    },

    /// SCID database version not supported
    ///
    /// The database uses a version number that isn't currently supported.
    /// The library typically supports version 400, but future versions
    /// may require updates to handle new features.
    #[error("Unsupported SCID version {version}, supported versions: {supported}")]
    UnsupportedVersion { version: u16, supported: u16 },
}

impl ScidError {
    /// Returns true if this error is recoverable (can skip and continue)
    ///
    /// Recoverable errors typically affect a single game and don't
    /// compromise the ability to read other games.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidError;
    ///
    /// let error = ScidError::MoveDecodeError { move_num: 5, byte: 0xFF, message: "unknown".into() };
    /// assert!(error.is_recoverable());
    ///
    /// let error = ScidError::FileNotFound { file: "db.si4".into() };
    /// assert!(!error.is_recoverable());
    /// ```
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ScidError::GameProcessError { .. }
                | ScidError::MoveDecodeError { .. }
                | ScidError::InvalidGameData { .. }
        )
    }

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

    // ===== Task 9A.6.1: Error Types and Messages Tests =====

    #[test]
    fn test_parse_error_display() {
        let err = ScidError::ParseError {
            file: PathBuf::from("test.si4"),
            offset: 0,
            message: "Invalid header magic".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("Parse error"));
        assert!(display.contains("Invalid header magic"));
        assert!(display.contains("offset 0"));
    }

    #[test]
    fn test_io_error_wrapping() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let scid_err: ScidError = io_err.into();

        let display = format!("{}", scid_err);
        assert!(display.contains("IO error") || display.contains("File not found"));
    }

    #[test]
    fn test_invalid_move_error() {
        let err = ScidError::MoveDecodeError {
            move_num: 12,
            byte: 0xFF,
            message: "unknown move".into(),
        };

        let display = format!("{}", err);
        assert!(display.contains("Invalid move"));
        assert!(display.contains("piece 12"));
        assert!(display.contains("0xFF"));
    }

    #[test]
    fn test_corrupt_database_error() {
        let err = ScidError::StructureError {
            message: "Game offset points beyond file end".into(),
        };

        let display = format!("{}", err);
        assert!(display.contains("Invalid game structure"));
        assert!(display.contains("Game offset"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ScidError>();
        assert_sync::<ScidError>();
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let scid_err: ScidError = io_err.into();

        assert!(scid_err.source().is_some());
    }

    // ===== Task 9A.6.2: Error Recoverability Tests =====

    #[test]
    fn test_recoverable_errors() {
        let recoverable_errors = vec![
            ScidError::GameProcessError {
                message: "test".into(),
                source: std::io::Error::new(std::io::ErrorKind::Other, "test"),
            },
            ScidError::MoveDecodeError {
                move_num: 5,
                byte: 0xFF,
                message: "unknown".into(),
            },
            ScidError::InvalidGameData("corrupted".into()),
        ];

        for err in recoverable_errors {
            assert!(
                err.is_recoverable(),
                "Error should be recoverable: {:?}",
                err
            );
        }
    }

    #[test]
    fn test_non_recoverable_errors() {
        let non_recoverable_errors = vec![
            ScidError::FileNotFound {
                file: "db.si4".into(),
            },
            ScidError::FileOpenError {
                file: "db.si4".into(),
                source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "locked"),
            },
            ScidError::InvalidGameIndex(100),
            ScidError::TooManyErrors { count: 10, max: 5 },
            ScidError::UnsupportedVersion {
                version: 500,
                supported: 400,
            },
        ];

        for err in non_recoverable_errors {
            assert!(
                !err.is_recoverable(),
                "Error should not be recoverable: {:?}",
                err
            );
        }
    }

    // ===== Task 9A.6.3: Error Construction Helpers Tests =====

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
    fn test_invalid_format_constructor() {
        let err = ScidError::invalid_format("wrong magic number");
        assert!(matches!(err, ScidError::InvalidFormat(_)));
        assert!(err.to_string().contains("wrong magic number"));
    }

    #[test]
    fn test_validation_constructor() {
        let err = ScidError::validation("ELO rating exceeds 4095");
        assert!(matches!(err, ScidError::Validation(_)));
        assert!(err.to_string().contains("ELO rating exceeds 4095"));
    }

    #[test]
    fn test_game_process_error_display() {
        let err = ScidError::GameProcessError {
            message: "Test processing error".into(),
            source: std::io::Error::new(std::io::ErrorKind::Other, "test"),
        };
        assert!(err.to_string().contains("Game processing error"));
        assert!(err.to_string().contains("Test processing error"));
    }

    #[test]
    fn test_structure_error_display() {
        let err = ScidError::StructureError {
            message: "corrupted game header".into(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid game structure: corrupted game header"
        );
    }

    #[test]
    fn test_encoding_error_display() {
        let err = ScidError::Encoding("Invalid UTF-8 sequence".into());
        assert_eq!(err.to_string(), "Encoding error: Invalid UTF-8 sequence");
    }

    #[test]
    fn test_validation_error_display() {
        let err = ScidError::Validation("Date month exceeds 12");
        assert_eq!(err.to_string(), "Validation error: Date month exceeds 12");
    }

    #[test]
    fn test_unsupported_error_display() {
        let err = ScidError::Unsupported("Chess960 format");
        assert_eq!(err.to_string(), "Unsupported feature: Chess960 format");
    }

    #[test]
    fn test_file_not_found_display() {
        let err = ScidError::FileNotFound {
            file: "db.si4".into(),
        };
        assert_eq!(err.to_string(), "File not found: db.si4");
    }

    #[test]
    fn test_file_open_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "locked");
        let err = ScidError::FileOpenError {
            file: "db.si4".into(),
            source: io_err,
        };
        assert!(err.to_string().contains("Failed to open file db.si4"));
        assert!(err.to_string().contains("locked"));
    }

    #[test]
    fn test_unsupported_version_error_display() {
        let err = ScidError::UnsupportedVersion {
            version: 500,
            supported: 400,
        };
        assert!(err.to_string().contains("Unsupported SCID version 500"));
        assert!(err.to_string().contains("supported versions: 400"));
    }
}
