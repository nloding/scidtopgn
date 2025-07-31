//! SCID to PGN Converter Library
//! 
//! This library provides functionality to read SCID chess databases and convert them to PGN format.
//! Major fixes implemented include proper date parsing and name extraction.

pub mod scid;
pub mod pgn;

pub use scid::{ScidDatabase, ScidHeader, GameIndex, IndexFile};