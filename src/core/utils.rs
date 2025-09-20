use std::io::Read;
use crate::core::error::Result;

/// Read a single byte from the reader
pub fn read_u8(reader: &mut impl Read) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Read a 2-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:305-313 ReadTwoBytes() implementation
pub fn read_u16_be(reader: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    let result = u16::from_be_bytes(buf);
    Ok(result)
}

/// Read a 3-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:325-334 ReadThreeBytes() implementation
pub fn read_u24_be(reader: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)?;
    // Big-endian: MSB first, LSB last (opposite of little-endian)
    let result = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
    Ok(result)
}

/// Read a 4-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:349-361 ReadFourBytes() implementation
pub fn read_u32_be(reader: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let result = u32::from_be_bytes(buf);
    Ok(result)
}

/// Read a null-terminated string of fixed length
pub fn read_string(reader: &mut impl Read, len: usize) -> Result<String> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    // Find first null byte and truncate there
    if let Some(null_pos) = buf.iter().position(|&b| b == 0) {
        buf.truncate(null_pos);
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u8() {
        let data = [0x42u8];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u8(&mut cursor).unwrap(), 0x42);
    }

    #[test]
    fn test_read_u16_be() {
        let data = [0x12u8, 0x34u8];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u16_be(&mut cursor).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u24_be() {
        let data = [0x12u8, 0x34u8, 0x56u8];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u24_be(&mut cursor).unwrap(), 0x123456);
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x12u8, 0x34u8, 0x56u8, 0x78u8];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u32_be(&mut cursor).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_string() {
        let data = [b'H', b'e', b'l', b'l', b'o', 0, b'W', b'o', b'r', b'l', b'd'];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_string(&mut cursor, 11).unwrap(), "Hello");
    }

    #[test]
    fn test_read_string_no_null() {
        let data = [b'H', b'e', b'l', b'l', b'o'];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_string(&mut cursor, 5).unwrap(), "Hello");
    }
}
