//! Low-level parsing utilities
//!
//! Binary parsing helpers and encoding support for SCID format.
//!
//! # Modules
//!
//! - `binary` - Binary data parsing helpers
//! - `encoding` - Character encoding support
//! - `byte_stream` - Sequential byte stream for multi-byte moves
//! - `position` - SCID position wrapper with piece tracking
//! - `decoder` - Piece-specific move decoders
//! - `move_decoder` - High-level move decoder

pub mod binary;
pub mod byte_stream;
pub mod decoder;
pub mod encoding;
pub mod move_decoder;
pub mod position;

// Re-exports for convenience
pub use byte_stream::ByteStream;
pub use decoder::DecodedMove;
pub use move_decoder::ScidMoveDecoder;
pub use position::{PieceNumberMapping, ScidPosition};
