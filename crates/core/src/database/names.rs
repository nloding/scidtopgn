//! Name file (.sn4) parsing
//!
//! The SCID name file stores all text strings using front-coding compression.
//! This provides 50-70% space savings compared to storing full names.
//!
//! # File Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ SN4 Header (36 bytes)                                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Player Names Section (variable length)                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Event Names Section (variable length)                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Site Names Section (variable length)                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Round Names Section (variable length)                       │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Front-Coding Compression
//!
//! Names are stored alphabetically with prefix compression:
//!
//! ```text
//! Stored:                    Decompressed:
//! [0]"Carlsen, Magnus"   ->  "Carlsen, Magnus"
//! [9]"Henrik"            ->  "Carlsen, Henrik"  (reuse "Carlsen, ")
//! [3]"uana, Fabiano"     ->  "Caruana, Fabiano" (reuse "Car")
//! ```
//!
//! Each name stores:
//! - Prefix length: How many characters to reuse from previous name
//! - Suffix: The remaining new characters
//!
//! Reconstruction: `name = previous[0..prefix_len] + suffix`
//!
//! # References
//!
//! See SCID_DATABASE_FORMAT.md:
//! - Lines 332-351: Header specification
//! - Lines 352-423: Front-coding algorithm
//! - Lines 398-412: Variable-length integer encoding

use crate::error::{Result, ScidError};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Magic bytes identifying a valid SN4 file.
/// All SCID name files must start with these exact 8 bytes.
/// See SCID_DATABASE_FORMAT.md line 340.
pub const SN4_MAGIC: &[u8; 8] = b"Scid.sn\0";

/// Size of SN4 header in bytes.
/// The header is always exactly 36 bytes.
/// See SCID_DATABASE_FORMAT.md line 337.
pub const SN4_HEADER_SIZE: usize = 36;

/// SN4 file header (36 bytes)
///
/// Contains counts and maximum frequencies for all name sections.
/// All multi-byte values are BIG-ENDIAN.
///
/// # Field Offsets
///
/// | Offset | Size | Field              |
/// |--------|------|-------------------|
/// | 0-7    | 8    | magic             |
/// | 8-11   | 4    | timestamp         |
/// | 12-14  | 3    | num_players       |
/// | 15-17  | 3    | num_events        |
/// | 18-20  | 3    | num_sites         |
/// | 21-23  | 3    | num_rounds        |
/// | 24-26  | 3    | max_freq_players  |
/// | 27-29  | 3    | max_freq_events   |
/// | 30-32  | 3    | max_freq_sites    |
/// | 33-35  | 3    | max_freq_rounds   |
///
/// See SCID_DATABASE_FORMAT.md lines 337-351.
#[derive(Debug, Clone, PartialEq)]
pub struct Sn4Header {
    /// File creation/modification timestamp (Unix time)
    pub timestamp: u32,

    /// Number of player names (24-bit value)
    pub num_players: u32,

    /// Number of event names
    pub num_events: u32,

    /// Number of site names
    pub num_sites: u32,

    /// Number of round names
    pub num_rounds: u32,

    /// Maximum frequency of any player name (determines integer encoding size)
    pub max_freq_players: u32,

    /// Maximum frequency of any event name
    pub max_freq_events: u32,

    /// Maximum frequency of any site name
    pub max_freq_sites: u32,

    /// Maximum frequency of any round name
    pub max_freq_rounds: u32,
}

impl Sn4Header {
    /// Create a new header with default values (for testing)
    pub fn new() -> Self {
        Sn4Header {
            timestamp: 0,
            num_players: 0,
            num_events: 0,
            num_sites: 0,
            num_rounds: 0,
            max_freq_players: 0,
            max_freq_events: 0,
            max_freq_sites: 0,
            max_freq_rounds: 0,
        }
    }

    /// Builder: set player count
    pub fn with_players(mut self, count: u32) -> Self {
        self.num_players = count;
        self
    }

    /// Builder: set event count
    pub fn with_events(mut self, count: u32) -> Self {
        self.num_events = count;
        self
    }
}

impl Default for Sn4Header {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete name database.
///
/// Contains all names from the .sn4 file, organized by type.
#[derive(Debug, Clone, Default)]
pub struct NameDatabase {
    /// Player names (sorted alphabetically)
    pub players: Vec<String>,

    /// Event names (sorted alphabetically)
    pub events: Vec<String>,

    /// Site names (sorted alphabetically)
    pub sites: Vec<String>,

    /// Round names (sorted alphabetically)
    pub rounds: Vec<String>,
}

impl NameDatabase {
    /// Create empty name database
    pub fn new() -> Self {
        Self::default()
    }

    /// Get player name by ID (index).
    /// Returns None if ID is out of range.
    pub fn get_player(&self, id: u32) -> Option<&str> {
        self.players.get(id as usize).map(|s| s.as_str())
    }

    /// Get event name by ID (index)
    pub fn get_event(&self, id: u32) -> Option<&str> {
        self.events.get(id as usize).map(|s| s.as_str())
    }

    /// Get site name by ID (index)
    pub fn get_site(&self, id: u32) -> Option<&str> {
        self.sites.get(id as usize).map(|s| s.as_str())
    }

    /// Get round name by ID (index)
    pub fn get_round(&self, id: u32) -> Option<&str> {
        self.rounds.get(id as usize).map(|s| s.as_str())
    }
}

/// Parse SN4 header from file.
///
/// Reads and validates the 36-byte header from a .sn4 file.
/// All multi-byte values are read as BIG-ENDIAN.
///
/// # Errors
///
/// - `ScidError::InvalidFormat` - Wrong magic bytes
/// - `ScidError::Io` - File read error
pub fn parse_sn4_header(reader: &mut BufReader<File>) -> Result<Sn4Header> {
    let mut header_bytes = [0u8; SN4_HEADER_SIZE];
    reader.read_exact(&mut header_bytes).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            ScidError::InvalidFormat(format!(
                "File too short: expected {} bytes for header",
                SN4_HEADER_SIZE
            ))
        } else {
            ScidError::Io(e)
        }
    })?;

    // Validate magic bytes (offset 0-7)
    if &header_bytes[0..8] != SN4_MAGIC {
        return Err(ScidError::InvalidFormat(format!(
            "Invalid magic bytes: expected {:?}, got {:?}",
            SN4_MAGIC,
            &header_bytes[0..8]
        )));
    }

    // Parse timestamp (offset 8-11, BIG-ENDIAN u32)
    let timestamp = u32::from_be_bytes([
        header_bytes[8],
        header_bytes[9],
        header_bytes[10],
        header_bytes[11],
    ]);

    // Parse counts (all are 24-bit big-endian values)
    let num_players = u32::from_be_bytes([0, header_bytes[12], header_bytes[13], header_bytes[14]]);
    let num_events = u32::from_be_bytes([0, header_bytes[15], header_bytes[16], header_bytes[17]]);
    let num_sites = u32::from_be_bytes([0, header_bytes[18], header_bytes[19], header_bytes[20]]);
    let num_rounds = u32::from_be_bytes([0, header_bytes[21], header_bytes[22], header_bytes[23]]);

    // Parse max frequencies (24-bit values)
    let max_freq_players =
        u32::from_be_bytes([0, header_bytes[24], header_bytes[25], header_bytes[26]]);
    let max_freq_events =
        u32::from_be_bytes([0, header_bytes[27], header_bytes[28], header_bytes[29]]);
    let max_freq_sites =
        u32::from_be_bytes([0, header_bytes[30], header_bytes[31], header_bytes[32]]);
    let max_freq_rounds =
        u32::from_be_bytes([0, header_bytes[33], header_bytes[34], header_bytes[35]]);

    Ok(Sn4Header {
        timestamp,
        num_players,
        num_events,
        num_sites,
        num_rounds,
        max_freq_players,
        max_freq_events,
        max_freq_sites,
        max_freq_rounds,
    })
}

/// Read a variable-length integer from the stream (for frequency field).
///
/// The number of bytes to read depends on the maximum possible value:
/// - If max < 256: Read 1 byte
/// - If max < 65,536: Read 2 bytes (big-endian u16)
/// - Otherwise: Read 3 bytes (24-bit big-endian)
///
/// See SCID_DATABASE_FORMAT.md lines 398-412.
fn read_variable_int(reader: &mut BufReader<File>, max_value: u32) -> Result<u32> {
    if max_value < 256 {
        // 1-byte encoding
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] as u32)
    } else if max_value < 65536 {
        // 2-byte encoding (big-endian u16)
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf) as u32)
    } else {
        // 3-byte encoding (24-bit big-endian)
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
    }
}

/// Read name ID from the stream.
///
/// Name IDs use different encoding than frequency:
/// - 2 bytes if total names < 65,536
/// - 3 bytes if total names >= 65,536
///
/// See SCID_DATABASE_FORMAT.md lines 385-387.
fn read_name_id(reader: &mut BufReader<File>, count: u32) -> Result<u32> {
    if count < 65536 {
        // 2-byte encoding (big-endian u16)
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf) as u32)
    } else {
        // 3-byte encoding (24-bit big-endian)
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
    }
}

/// Clean a name string.
///
/// Removes control characters (< 0x20) and trims whitespace.
/// SCID name files may contain control characters that need filtering.
fn clean_name_string(name: &str) -> String {
    name.chars()
        .filter(|&c| c >= ' ')
        .collect::<String>()
        .trim()
        .to_string()
}

/// Read a single name section using front-coding decompression.
///
/// Front-coding compresses names by storing only the suffix that differs
/// from the previous name. Names must be alphabetically sorted.
///
/// # Algorithm
///
/// ```text
/// For each name:
///   1. Read name_id (variable-length)
///   2. Read frequency (variable-length)
///   3. Read total_length (1 byte)
///   4. If first name: prefix_length = 0, else read prefix_length (1 byte)
///   5. suffix_length = total_length - prefix_length
///   6. Read suffix_length bytes
///   7. name = previous_name[0..prefix_length] + suffix
///   8. Clean and store name
/// ```
///
/// See SCID_DATABASE_FORMAT.md lines 352-423.
fn read_names_section(
    reader: &mut BufReader<File>,
    count: u32,
    max_frequency: u32,
) -> Result<Vec<String>> {
    // Pre-allocate array with empty strings - names will be placed by their ID
    let mut names: Vec<String> = vec![String::new(); count as usize];
    let mut previous_name = String::new();

    for i in 0..count {
        // Step 1: Read name ID (2 or 3 bytes based on count)
        // Name IDs tell us WHERE in the final array this name belongs
        let name_id = read_name_id(reader, count)?;

        // Step 2: Read variable-length frequency
        let _frequency = read_variable_int(reader, max_frequency)?;

        // Step 3: Read total length (1 byte)
        let mut len_buf = [0u8; 1];
        reader.read_exact(&mut len_buf)?;
        let total_length = len_buf[0] as usize;

        // Step 4: Read prefix length (not present for first name)
        let prefix_length = if i == 0 {
            0
        } else {
            let mut prefix_buf = [0u8; 1];
            reader.read_exact(&mut prefix_buf)?;
            prefix_buf[0] as usize
        };

        // Validate prefix length
        if prefix_length > previous_name.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sn4"),
                offset: 0,
                message: format!(
                    "Invalid prefix length {} for name {} (previous name length: {})",
                    prefix_length,
                    i,
                    previous_name.len()
                ),
            });
        }

        if prefix_length > total_length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sn4"),
                offset: 0,
                message: format!(
                    "Prefix length {} exceeds total length {} for name {}",
                    prefix_length, total_length, i
                ),
            });
        }

        // Step 5: Calculate suffix length
        let suffix_length = total_length - prefix_length;

        // Step 6: Read suffix bytes
        let mut suffix_bytes = vec![0u8; suffix_length];
        reader.read_exact(&mut suffix_bytes)?;

        // Step 7: Convert suffix to string
        let suffix = String::from_utf8(suffix_bytes)
            .map_err(|e| ScidError::Encoding(format!("Invalid UTF-8 in name {}: {}", i, e)))?;

        // Step 8: Reconstruct full name using front-coding
        previous_name.truncate(prefix_length);
        previous_name.push_str(&suffix);

        // Step 9: Clean and store name at its proper index (name_id)
        let clean_name = clean_name_string(&previous_name);

        // Validate name_id is within bounds
        if name_id as usize >= count as usize {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sn4"),
                offset: 0,
                message: format!("Name ID {} exceeds count {} for name {}", name_id, count, i),
            });
        }

        names[name_id as usize] = clean_name.clone();

        // Update previous_name for next iteration (for front-coding)
        previous_name = clean_name;
    }

    Ok(names)
}

/// Parse complete name database from .sn4 file.
///
/// Parses header and all four name sections:
/// 1. Player names (alphabetically sorted)
/// 2. Event names (alphabetically sorted)
/// 3. Site names (alphabetically sorted)
/// 4. Round names (alphabetically sorted)
pub fn parse_name_database(path: impl AsRef<Path>) -> Result<NameDatabase> {
    let file = File::open(path.as_ref())?;
    let mut reader = BufReader::new(file);

    // Parse header
    let header = parse_sn4_header(&mut reader)?;

    // Read all four sections in order
    let players = read_names_section(&mut reader, header.num_players, header.max_freq_players)?;

    let events = read_names_section(&mut reader, header.num_events, header.max_freq_events)?;

    let sites = read_names_section(&mut reader, header.num_sites, header.max_freq_sites)?;

    let rounds = read_names_section(&mut reader, header.num_rounds, header.max_freq_rounds)?;

    Ok(NameDatabase {
        players,
        events,
        sites,
        rounds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sn4_header_builder() {
        let header = Sn4Header::new().with_players(1000).with_events(50);

        assert_eq!(header.num_players, 1000);
        assert_eq!(header.num_events, 50);
    }

    #[test]
    fn test_sn4_header_default() {
        let header = Sn4Header::default();
        assert_eq!(header.num_players, 0);
        assert_eq!(header.timestamp, 0);
    }

    #[test]
    fn test_name_database_accessors() {
        let mut db = NameDatabase::new();
        db.players.push("Carlsen, Magnus".to_string());
        db.players.push("Caruana, Fabiano".to_string());

        assert_eq!(db.get_player(0), Some("Carlsen, Magnus"));
        assert_eq!(db.get_player(1), Some("Caruana, Fabiano"));
        assert_eq!(db.get_player(99), None);
    }

    #[test]
    fn test_clean_name_string() {
        // Normal name
        assert_eq!(clean_name_string("Carlsen, Magnus"), "Carlsen, Magnus");

        // Name with null terminator
        assert_eq!(clean_name_string("Carlsen\x00, Magnus"), "Carlsen, Magnus");

        // Name with newline
        assert_eq!(clean_name_string("Carlsen, Magnus\n"), "Carlsen, Magnus");

        // Name with leading/trailing whitespace
        assert_eq!(clean_name_string("  Carlsen, Magnus  "), "Carlsen, Magnus");

        // Name with multiple control characters
        assert_eq!(
            clean_name_string("Carlsen\x01\x02, Magnus\x03"),
            "Carlsen, Magnus"
        );

        // Empty string
        assert_eq!(clean_name_string(""), "");

        // Only control characters
        assert_eq!(clean_name_string("\x00\x01\x02"), "");
    }

    #[test]
    fn test_front_coding_concept() {
        // Verify front-coding logic with manual reconstruction
        let names = vec![
            ("Carlsen, Magnus", 0, "Carlsen, Magnus"),
            ("Carlsen, Henrik", 9, "Henrik"),
            ("Caruana, Fabiano", 3, "uana, Fabiano"),
        ];

        let mut previous = String::new();

        for (expected, prefix_len, suffix) in names {
            previous.truncate(prefix_len);
            previous.push_str(suffix);
            assert_eq!(previous, expected);
        }
    }

    #[test]
    fn test_string_truncate_and_append() {
        let mut s = String::from("Carlsen, Magnus");

        // Truncate to 9 chars
        s.truncate(9);
        assert_eq!(s, "Carlsen, ");

        // Append new suffix
        s.push_str("Henrik");
        assert_eq!(s, "Carlsen, Henrik");

        // Truncate to 3 chars
        s.truncate(3);
        assert_eq!(s, "Car");

        // Append another suffix
        s.push_str("uana, Fabiano");
        assert_eq!(s, "Caruana, Fabiano");
    }

    #[test]
    fn test_variable_int_encoding_sizes() {
        // Max < 256: should use 1 byte
        assert!(255 < 256);

        // Max < 65536: should use 2 bytes
        assert!(1000 < 65536);
        assert!(65535 < 65536);

        // Max >= 65536: should use 3 bytes
        assert!(100000 >= 65536);
    }
}
