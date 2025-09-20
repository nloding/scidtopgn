// PGN Export Module
//
// This module provides standards-compliant PGN (Portable Game Notation) export
// functionality with full chess validation integration. It ensures that only
// valid, chess-legal games are exported to PGN format.

pub mod annotation_formatter;
pub mod exporter;
pub mod header;
pub mod standards_compliance;
pub mod variation_formatter;

// Re-export key types for easier access
pub use exporter::*;

pub use header::PgnHeader;
