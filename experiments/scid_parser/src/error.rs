// Enhanced Error Handling for SCID Parser with Shakmaty Integration
//
// This module provides comprehensive error handling using thiserror for clean
// error propagation and integration with shakmaty's error types. It creates
// a unified error system that can handle both SCID parsing errors and chess
// logic errors from shakmaty.

use thiserror::Error;

/// Comprehensive error type for SCID parsing and chess operations
#[derive(Error, Debug)]
pub enum ScidError {
    /// Standard IO errors (file reading, writing, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Invalid SCID file format errors
    #[error("Invalid SCID format: {message}")]
    InvalidFormat { message: String },
    
    /// Parse errors with specific location information
    #[error("Parse error at offset {offset}: {message}")]
    ParseError { offset: usize, message: String },
    
    /// Chess position errors from shakmaty
    #[error("Chess position error: {0}")]
    Chess(#[from] shakmaty::PositionError<shakmaty::Chess>),
    
    /// Invalid move errors from shakmaty
    #[error("Invalid move: {0}")]
    InvalidMove(#[from] shakmaty::PlayError<shakmaty::Chess>),
    
    /// SCID to Shakmaty conversion errors
    #[error("SCID to Shakmaty conversion error: {message}")]
    ConversionError { message: String },
    
    /// Name database lookup errors
    #[error("Name lookup error: {message}")]
    NameLookupError { message: String },
    
    /// Game index out of bounds
    #[error("Game index {index} out of bounds (max: {max})")]
    GameIndexOutOfBounds { index: usize, max: usize },
    
    /// Invalid game data
    #[error("Invalid game data: {message}")]
    InvalidGameData { message: String },
    
    /// Missing required data
    #[error("Missing required data: {message}")]
    MissingData { message: String },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ScidError>;

impl ScidError {
    /// Create a new invalid format error
    pub fn invalid_format(message: impl Into<String>) -> Self {
        ScidError::InvalidFormat {
            message: message.into(),
        }
    }
    
    /// Create a new parse error with offset
    pub fn parse_error(offset: usize, message: impl Into<String>) -> Self {
        ScidError::ParseError {
            offset,
            message: message.into(),
        }
    }
    
    /// Create a new conversion error
    pub fn conversion_error(message: impl Into<String>) -> Self {
        ScidError::ConversionError {
            message: message.into(),
        }
    }
    
    /// Create a new name lookup error
    pub fn name_lookup_error(message: impl Into<String>) -> Self {
        ScidError::NameLookupError {
            message: message.into(),
        }
    }
    
    /// Create a new game index out of bounds error
    pub fn game_index_out_of_bounds(index: usize, max: usize) -> Self {
        ScidError::GameIndexOutOfBounds { index, max }
    }
    
    /// Create a new invalid game data error
    pub fn invalid_game_data(message: impl Into<String>) -> Self {
        ScidError::InvalidGameData {
            message: message.into(),
        }
    }
    
    /// Create a new missing data error
    pub fn missing_data(message: impl Into<String>) -> Self {
        ScidError::MissingData {
            message: message.into(),
        }
    }
}

/// Extension trait for Results to add convenient error creation methods
pub trait ResultExt<T> {
    /// Convert to ScidError::InvalidFormat with message
    fn invalid_format(self, message: impl Into<String>) -> Result<T>;
    
    /// Convert to ScidError::ParseError with offset and message
    fn parse_error(self, offset: usize, message: impl Into<String>) -> Result<T>;
    
    /// Convert to ScidError::ConversionError with message
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

/// Extension trait for Option to add convenient error creation methods
pub trait OptionExt<T> {
    /// Convert None to ScidError::MissingData with message
    fn missing_data(self, message: impl Into<String>) -> Result<T>;
    
    /// Convert None to ScidError::InvalidFormat with message
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