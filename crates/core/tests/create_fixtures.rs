//! Test fixture generator
//!
//! This creates minimal SCID databases for integration testing.
//! Run with: cargo test --test create_fixtures -- --ignored

use byteorder::{BigEndian, WriteBytesExt};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
#[ignore]
fn generate_minimal_fixture() {
    let fixture_dir = Path::new("tests/fixtures/minimal");
    fs::create_dir_all(fixture_dir).unwrap();

    let mut si4 = File::create(fixture_dir.join("minimal.si4")).unwrap();

    si4.write_all(b"\x53\x63\x69\x64\x20\x44\x42\x00").unwrap();
    si4.write_u32::<BigEndian>(0x00040000).unwrap();
    si4.write_u32::<BigEndian>(0).unwrap();
    si4.write_u32::<BigEndian>(1).unwrap();
    si4.write_u32::<BigEndian>(0).unwrap();

    let zeros = vec![0u8; 256 - 24];
    si4.write_all(&zeros).unwrap();

    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u16::<BigEndian>(20).unwrap();

    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(1).unwrap();

    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();

    si4.write_u8(0x00).unwrap();

    let date = ((2024u32 & 0xFFF) << 9) | ((3u32 & 0x0F) << 5) | (15u32 & 0x1F);
    si4.write_u8((date >> 16) as u8).unwrap();
    si4.write_u16::<BigEndian>((date & 0xFFFF) as u16).unwrap();

    si4.write_u16::<BigEndian>(2800).unwrap();
    si4.write_u16::<BigEndian>(2750).unwrap();

    let remaining = vec![0u8; 46 - 27];
    si4.write_all(&remaining).unwrap();

    let mut sn4 = File::create(fixture_dir.join("minimal.sn4")).unwrap();

    sn4.write_u32::<BigEndian>(2).unwrap();
    sn4.write_u32::<BigEndian>(1).unwrap();
    sn4.write_u32::<BigEndian>(1).unwrap();
    sn4.write_u32::<BigEndian>(0).unwrap();

    let name = "Carlsen, Magnus";
    sn4.write_u8(((0 & 0x0F) | ((name.len() as u8) << 4)))
        .unwrap();
    sn4.write_all(name.as_bytes()).unwrap();

    let name2 = "Nakamura, Hikaru";
    sn4.write_u8(((0 & 0x0F) | ((name2.len() as u8) << 4)))
        .unwrap();
    sn4.write_all(name2.as_bytes()).unwrap();

    let event = "Test Tournament";
    sn4.write_u8(((0 & 0x0F) | ((event.len() as u8) << 4)))
        .unwrap();
    sn4.write_all(event.as_bytes()).unwrap();

    let site = "Online";
    sn4.write_u8(((0 & 0x0F) | ((site.len() as u8) << 4)))
        .unwrap();
    sn4.write_all(site.as_bytes()).unwrap();

    let mut sg4 = File::create(fixture_dir.join("minimal.sg4")).unwrap();

    sg4.write_u8(0x00).unwrap();

    sg4.write_u8(12).unwrap();
    sg4.write_u8(0x02).unwrap();

    sg4.write_u8(28).unwrap();
    sg4.write_u8(0x02).unwrap();

    sg4.write_u8(6).unwrap();
    sg4.write_u8(0x14).unwrap();

    sg4.write_u8(22).unwrap();
    sg4.write_u8(0x15).unwrap();

    sg4.write_u8(0xFF).unwrap();
    sg4.write_u8(0xFF).unwrap();

    println!("Created minimal fixture at tests/fixtures/minimal/");
    println!("  - minimal.si4 (header + 1 game entry)");
    println!("  - minimal.sn4 (2 players, 1 event, 1 site)");
    println!("  - minimal.sg4 (1 game with 4 moves)");
}

#[test]
#[ignore]
fn generate_expected_pgn() {
    let expected = r#"[Event "Test Tournament"]
[Site "Online"]
[Date "2024.03.15"]
[Round "?"]
[White "Carlsen, Magnus"]
[Black "Nakamura, Hikaru"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0
"#;

    let output_path = Path::new("tests/fixtures/expected/minimal.pgn");
    fs::create_dir_all(output_path.parent().unwrap()).unwrap();

    let mut file = File::create(output_path).unwrap();
    file.write_all(expected.as_bytes()).unwrap();

    println!("Created expected output at tests/fixtures/expected/minimal.pgn");
}
