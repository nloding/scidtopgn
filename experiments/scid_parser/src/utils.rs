use std::io::{self, Read};
use crate::error::{Result, ScidError};

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
    Ok(result)
}

/// Read a 3-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:325-334 ReadThreeBytes() implementation
pub fn read_u24_be(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)?;
    // Big-endian: MSB first, LSB last (opposite of little-endian)
    let result = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
    Ok(result)
}


/// Read a 4-byte big-endian unsigned integer (SCID format)
/// Based on SCID's mfile.cpp:349-361 ReadFourBytes() implementation
pub fn read_u32_be(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let result = u32::from_be_bytes(buf);
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

// Enhanced versions using the new error system
// These provide better error messages and integration with shakmaty errors

/// Read a single byte with enhanced error reporting
pub fn read_u8_enhanced(reader: &mut impl Read, context: &str) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)
        .map_err(|e| ScidError::parse_error(0, format!("Failed to read u8 for {}: {}", context, e)))?;
    Ok(buf[0])
}

/// Read a 2-byte big-endian unsigned integer with enhanced error reporting
pub fn read_u16_be_enhanced(reader: &mut impl Read, context: &str) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)
        .map_err(|e| ScidError::parse_error(0, format!("Failed to read u16 for {}: {}", context, e)))?;
    Ok(u16::from_be_bytes(buf))
}

/// Read a 3-byte big-endian unsigned integer with enhanced error reporting
pub fn read_u24_be_enhanced(reader: &mut impl Read, context: &str) -> Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)
        .map_err(|e| ScidError::parse_error(0, format!("Failed to read u24 for {}: {}", context, e)))?;
    
    // Validate the value is within expected range for 24-bit
    let result = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
    if result > 0xFFFFFF {
        return Err(ScidError::invalid_format(
            format!("Invalid 24-bit value {} for {}", result, context)
        ));
    }
    
    Ok(result)
}

/// Read a 4-byte big-endian unsigned integer with enhanced error reporting
pub fn read_u32_be_enhanced(reader: &mut impl Read, context: &str) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)
        .map_err(|e| ScidError::parse_error(0, format!("Failed to read u32 for {}: {}", context, e)))?;
    Ok(u32::from_be_bytes(buf))
}

/// Read a null-terminated string with enhanced error reporting and validation
pub fn read_string_enhanced(reader: &mut impl Read, len: usize, context: &str) -> Result<String> {
    if len == 0 {
        return Err(ScidError::invalid_format(
            format!("Invalid string length 0 for {}", context)
        ));
    }
    
    if len > 1024 {
        return Err(ScidError::invalid_format(
            format!("String length {} too large for {}", len, context)
        ));
    }
    
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)
        .map_err(|e| ScidError::parse_error(0, format!("Failed to read string for {}: {}", context, e)))?;
    
    // Find first null byte and truncate there
    if let Some(null_pos) = buf.iter().position(|&b| b == 0) {
        buf.truncate(null_pos);
    }
    
    // Validate UTF-8 encoding
    match String::from_utf8(buf) {
        Ok(s) => Ok(s),
        Err(e) => {
            // Fall back to lossy conversion but report the error
            let lossy = String::from_utf8_lossy(&e.into_bytes()).to_string();
            Err(ScidError::invalid_format(
                format!("Invalid UTF-8 in string for {}, using lossy conversion: {}", context, lossy)
            ))
        }
    }
}

/// Read bytes at a specific offset with enhanced error reporting
pub fn read_bytes_at_offset_enhanced(
    reader: &mut impl Read, 
    offset: usize, 
    len: usize, 
    context: &str
) -> Result<Vec<u8>> {
    // Note: This is a placeholder implementation
    // In a real scenario, we'd need seekable readers for offset operations
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)
        .map_err(|e| ScidError::parse_error(
            offset, 
            format!("Failed to read {} bytes at offset {} for {}: {}", len, offset, context, e)
        ))?;
    Ok(buf)
}

/// Validate magic bytes with enhanced error reporting
pub fn validate_magic_bytes(expected: &[u8], actual: &[u8], file_type: &str) -> Result<()> {
    if expected.len() != actual.len() {
        return Err(ScidError::invalid_format(
            format!("Magic bytes length mismatch for {}: expected {} bytes, got {}", 
                   file_type, expected.len(), actual.len())
        ));
    }
    
    if expected != actual {
        return Err(ScidError::invalid_format(
            format!("Invalid magic bytes for {}: expected {:?}, got {:?}", 
                   file_type, expected, actual)
        ));
    }
    
    Ok(())
}

/// Convert io::Result to our enhanced Result type with context
pub fn io_result_to_scid_result<T>(
    result: io::Result<T>, 
    offset: usize, 
    context: &str
) -> Result<T> {
    result.map_err(|e| ScidError::parse_error(offset, format!("{}: {}", context, e)))
}

// Backward compatibility wrappers
// These convert from enhanced error types back to io::Result for existing code

/// Convert enhanced u8 read to io::Result for backward compatibility
pub fn read_u8_compat(reader: &mut impl Read, context: &str) -> io::Result<u8> {
    read_u8_enhanced(reader, context)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read u8 for {}", context)))
}

/// Convert enhanced u16 read to io::Result for backward compatibility
pub fn read_u16_be_compat(reader: &mut impl Read, context: &str) -> io::Result<u16> {
    read_u16_be_enhanced(reader, context)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read u16 for {}", context)))
}

/// Convert enhanced u24 read to io::Result for backward compatibility
pub fn read_u24_be_compat(reader: &mut impl Read, context: &str) -> io::Result<u32> {
    read_u24_be_enhanced(reader, context)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read u24 for {}", context)))
}

/// Convert enhanced u32 read to io::Result for backward compatibility
pub fn read_u32_be_compat(reader: &mut impl Read, context: &str) -> io::Result<u32> {
    read_u32_be_enhanced(reader, context)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read u32 for {}", context)))
}

/// Convert enhanced string read to io::Result for backward compatibility
pub fn read_string_compat(reader: &mut impl Read, len: usize, context: &str) -> io::Result<String> {
    read_string_enhanced(reader, len, context)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read string for {}", context)))
}