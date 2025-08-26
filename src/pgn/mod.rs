// PGN Export Module
//
// This module provides standards-compliant PGN (Portable Game Notation) export
// functionality with full chess validation integration. It ensures that only
// valid, chess-legal games are exported to PGN format.

pub mod exporter;
pub mod variation_formatter;
pub mod annotation_formatter;
pub mod standards_compliance;

// Re-export key types for easier access
pub use exporter::*;
pub use variation_formatter::VariationFormatter;
pub use annotation_formatter::AnnotationFormatter;
pub use standards_compliance::PgnStandardsChecker;