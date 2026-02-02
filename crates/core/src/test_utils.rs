//! Test utilities for creating test fixtures
//!
//! This module provides helper functions to generate valid SCID data
//! for testing purposes. Not compiled in release builds.

#![cfg(test)]

use byteorder::{BigEndian, WriteBytesExt};

/// Creates a minimal valid .si4 header
///
/// See SCID_DATABASE_FORMAT.md lines 75-127 for header specification
pub fn create_test_si4_header(game_count: u32) -> Vec<u8> {
    let mut header = Vec::new();

    header.extend_from_slice(b"\x53\x63\x69\x64\x20\x44\x42\x00");

    header.write_u32::<BigEndian>(0x00040000).unwrap();

    header.write_u32::<BigEndian>(0).unwrap();

    header.write_u32::<BigEndian>(game_count).unwrap();

    header.write_u32::<BigEndian>(0).unwrap();

    header.resize(256, 0);

    header
}

/// Creates a test game index entry
///
/// See SCID_DATABASE_FORMAT.md lines 129-209 for index entry specification
pub fn create_test_game_entry(
    white_id: u32,
    black_id: u32,
    event_id: u32,
    site_id: u32,
    round_id: u32,
    offset: u32,
    length: u16,
) -> Vec<u8> {
    let mut entry = Vec::new();

    entry.write_u8((offset >> 16) as u8).unwrap();
    entry
        .write_u16::<BigEndian>((offset & 0xFFFF) as u16)
        .unwrap();

    entry.write_u16::<BigEndian>(length).unwrap();

    entry.write_u8((white_id >> 16) as u8).unwrap();
    entry
        .write_u16::<BigEndian>((white_id & 0xFFFF) as u16)
        .unwrap();

    entry.write_u8((black_id >> 16) as u8).unwrap();
    entry
        .write_u16::<BigEndian>((black_id & 0xFFFF) as u16)
        .unwrap();

    entry.write_u8((event_id >> 16) as u8).unwrap();
    entry
        .write_u16::<BigEndian>((event_id & 0xFFFF) as u16)
        .unwrap();

    entry.write_u8((site_id >> 16) as u8).unwrap();
    entry
        .write_u16::<BigEndian>((site_id & 0xFFFF) as u16)
        .unwrap();

    entry.write_u8((round_id >> 16) as u8).unwrap();
    entry
        .write_u16::<BigEndian>((round_id & 0xFFFF) as u16)
        .unwrap();

    entry.resize(46, 0);

    entry
}

/// Creates a test name file entry with front-coding
///
/// See SCID_DATABASE_FORMAT.md lines 315-470 for name file specification
pub fn create_test_name_entry(name: &str, previous_name: &str) -> Vec<u8> {
    let mut entry = Vec::new();

    let common_len = name
        .chars()
        .zip(previous_name.chars())
        .take_while(|(a, b)| a == b)
        .count();

    let new_chars = &name[common_len..];
    let length_byte = ((common_len as u8) & 0x0F) | (((new_chars.len() as u8) & 0x0F) << 4);

    entry.push(length_byte);
    entry.extend_from_slice(new_chars.as_bytes());

    entry
}

/// Creates a minimal valid game data block
///
/// Returns bytes for a game with just 1.e4 e5 2.Nf3
pub fn create_test_game_data() -> Vec<u8> {
    let mut data = Vec::new();

    data.push(0x00);

    data.push(12);
    data.push(0x02);

    data.push(12);
    data.push(0x02);

    data.push(6);
    data.push(0x14);

    data.push(0xFF);
    data.push(0xFF);

    data
}

/// Creates a complete minimal test database in memory
pub struct TestDatabase {
    pub si4_data: Vec<u8>,
    pub sn4_data: Vec<u8>,
    pub sg4_data: Vec<u8>,
}

impl TestDatabase {
    pub fn new_single_game() -> Self {
        let mut si4_data = create_test_si4_header(1);
        let game_data = create_test_game_data();

        si4_data.extend(create_test_game_entry(
            1,
            2,
            1,
            1,
            0,
            0,
            game_data.len() as u16,
        ));

        let mut sn4_data = Vec::new();

        sn4_data.write_u32::<BigEndian>(3).unwrap();

        sn4_data.write_u32::<BigEndian>(2).unwrap();

        sn4_data.write_u32::<BigEndian>(1).unwrap();

        sn4_data.write_u32::<BigEndian>(1).unwrap();

        sn4_data.write_u32::<BigEndian>(0).unwrap();

        sn4_data.extend(create_test_name_entry("Carlsen, Magnus", ""));

        sn4_data.extend(create_test_name_entry(
            "Nakamura, Hikaru",
            "Carlsen, Magnus",
        ));

        sn4_data.extend(create_test_name_entry("Test Tournament", ""));

        sn4_data.extend(create_test_name_entry("Online", ""));

        Self {
            si4_data,
            sn4_data,
            sg4_data: game_data,
        }
    }
}

/// Helper to create temporary test files
pub fn write_test_database(db: &TestDatabase) -> std::io::Result<tempfile::TempDir> {
    use std::fs::File;
    use std::io::Write;

    let dir = tempfile::tempdir()?;

    let si4_path = dir.path().join("test.si4");
    let mut si4_file = File::create(si4_path)?;
    si4_file.write_all(&db.si4_data)?;

    let sn4_path = dir.path().join("test.sn4");
    let mut sn4_file = File::create(sn4_path)?;
    sn4_file.write_all(&db.sn4_data)?;

    let sg4_path = dir.path().join("test.sg4");
    let mut sg4_file = File::create(sg4_path)?;
    sg4_file.write_all(&db.sg4_data)?;

    Ok(dir)
}
