use std::io::{self, Read};

/// Read a single byte from the reader
pub fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}


/// Read a 2-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:305-313 ReadTwoBytes() implementation
pub fn read_u16_be(reader: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    let result = u16::from_be_bytes(buf);
    println!("DEBUG: read_u16_be - bytes: [{:02x}, {:02x}] = {}", buf[0], buf[1], result);
    Ok(result)
}

/// Read a 3-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:325-334 ReadThreeBytes() implementation
pub fn read_u24_be(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)?;
    // Big-endian: MSB first, LSB last (opposite of little-endian)
    let result = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
    println!("DEBUG: read_u24_be - bytes: [{:02x}, {:02x}, {:02x}] = {}", buf[0], buf[1], buf[2], result);
    Ok(result)
}


/// Read a 4-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:349-361 ReadFourBytes() implementation
pub fn read_u32_be(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let result = u32::from_be_bytes(buf);
    println!("DEBUG: read_u32_be - bytes: [{:02x}, {:02x}, {:02x}, {:02x}] = {}", 
        buf[0], buf[1], buf[2], buf[3], result);
    Ok(result)
}

/// Read a null-terminated string of fixed length
pub fn read_string(reader: &mut impl Read, len: usize) -> io::Result<String> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    // Find first null byte and truncate there
    if let Some(null_pos) = buf.iter().position(|&b| b == 0) {
        buf.truncate(null_pos);
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}