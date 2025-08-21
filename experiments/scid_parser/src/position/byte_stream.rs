// SCID-compatible byte stream reader
// Replicates functionality from scidvspc/src/bytebuf.h ByteBuffer class
//
// This module provides ByteBuffer-equivalent streaming functionality for handling
// variable-length SCID move encodings, specifically Queen diagonal moves which
// require 2-byte sequences.

/// SCID-compatible byte stream reader
/// Replicates functionality from scidvspc/src/bytebuf.h ByteBuffer class
pub struct ScidByteStream<'a> {
    /// Raw game data bytes
    buffer: &'a [u8],
    
    /// Current read position (equivalent to SCID's ReadPos)
    read_pos: usize,
    
    /// Total bytes available (equivalent to SCID's ByteCount)
    byte_count: usize,
    
    /// Error state tracking
    error_state: Option<String>,
}

impl<'a> ScidByteStream<'a> {
    /// Create new stream from game data
    /// Equivalent to SCID's ByteBuffer constructor with external buffer
    pub fn new(game_data: &'a [u8]) -> Self {
        ScidByteStream {
            buffer: game_data,
            read_pos: 0,
            byte_count: game_data.len(),
            error_state: None,
        }
    }
    
    /// Read next byte from stream
    /// Equivalent to SCID's ByteBuffer::GetByte()
    /// 
    /// SCID implementation from bytebuf.h:
    /// ```cpp
    /// GetByte () {
    ///     ASSERT(Current != NULL);
    ///     if (ReadPos >= ByteCount) { Err = ERROR_BufferRead; return 0; }
    ///     byte b = *Current;
    ///     Current++; ReadPos++;
    ///     return b;
    /// }
    /// ```
    pub fn get_byte(&mut self) -> Result<u8, String> {
        if self.read_pos >= self.byte_count {
            let error = "Buffer underrun - attempted to read beyond end".to_string();
            self.error_state = Some(error.clone());
            return Err(error);
        }
        
        let byte = self.buffer[self.read_pos];
        self.read_pos += 1;  // Advance position (equivalent to Current++; ReadPos++ in SCID)
        
        Ok(byte)
    }
    
    /// Peek at next byte without advancing position
    /// Not present in SCID's ByteBuffer but useful for our parsing logic
    pub fn peek_byte(&self) -> Option<u8> {
        if self.read_pos >= self.byte_count {
            None
        } else {
            Some(self.buffer[self.read_pos])
        }
    }
    
    /// Get current read position
    /// Equivalent to accessing SCID's ReadPos directly
    pub fn position(&self) -> usize {
        self.read_pos
    }
    
    /// Check if more bytes available
    /// Equivalent to checking (ReadPos < ByteCount) in SCID
    pub fn has_bytes(&self) -> bool {
        self.read_pos < self.byte_count
    }
    
    /// Get remaining byte count
    /// Equivalent to (ByteCount - ReadPos) in SCID
    pub fn bytes_remaining(&self) -> usize {
        self.byte_count.saturating_sub(self.read_pos)
    }
    
    /// Get total byte count
    /// Equivalent to SCID's GetByteCount()
    pub fn total_bytes(&self) -> usize {
        self.byte_count
    }
    
    /// Check if stream is in error state
    /// Equivalent to checking SCID's Err field
    pub fn has_error(&self) -> bool {
        self.error_state.is_some()
    }
    
    /// Get error state
    /// Equivalent to accessing SCID's Err field
    pub fn error(&self) -> Option<&String> {
        self.error_state.as_ref()
    }
    
    /// Reset error state
    /// Equivalent to setting SCID's Err = OK
    pub fn clear_error(&mut self) {
        self.error_state = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_byte_reading() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut stream = ScidByteStream::new(&data);
        
        assert_eq!(stream.get_byte().unwrap(), 0x12);
        assert_eq!(stream.position(), 1);
        assert_eq!(stream.bytes_remaining(), 3);
        
        assert_eq!(stream.get_byte().unwrap(), 0x34);
        assert_eq!(stream.position(), 2);
        assert_eq!(stream.bytes_remaining(), 2);
        
        assert!(stream.has_bytes());
        assert!(!stream.has_error());
    }
    
    #[test]
    fn test_buffer_underrun() {
        let data = vec![0x12];
        let mut stream = ScidByteStream::new(&data);
        
        // First read should succeed
        assert_eq!(stream.get_byte().unwrap(), 0x12);
        assert_eq!(stream.position(), 1);
        assert!(!stream.has_bytes());
        
        // Second read should fail with error
        let result = stream.get_byte();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Buffer underrun"));
        assert!(stream.has_error());
    }
    
    #[test]
    fn test_peek_functionality() {
        let data = vec![0x12, 0x34];
        let stream = ScidByteStream::new(&data);
        
        // Peek should not advance position
        assert_eq!(stream.peek_byte().unwrap(), 0x12);
        assert_eq!(stream.position(), 0); // Position unchanged
        
        // Multiple peeks should return same value
        assert_eq!(stream.peek_byte().unwrap(), 0x12);
        assert_eq!(stream.position(), 0);
    }
    
    #[test]
    fn test_empty_stream() {
        let data = vec![];
        let stream = ScidByteStream::new(&data);
        
        assert_eq!(stream.total_bytes(), 0);
        assert_eq!(stream.bytes_remaining(), 0);
        assert!(!stream.has_bytes());
        assert!(stream.peek_byte().is_none());
    }
    
    #[test]
    fn test_sequential_reading() {
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        let mut stream = ScidByteStream::new(&data);
        
        let mut read_bytes = Vec::new();
        while stream.has_bytes() {
            read_bytes.push(stream.get_byte().unwrap());
        }
        
        assert_eq!(read_bytes, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
        assert_eq!(stream.position(), 5);
        assert_eq!(stream.bytes_remaining(), 0);
    }
    
    #[test]
    fn test_error_state_management() {
        let data = vec![0x01];
        let mut stream = ScidByteStream::new(&data);
        
        // Initially no error
        assert!(!stream.has_error());
        assert!(stream.error().is_none());
        
        // Read valid byte
        stream.get_byte().unwrap();
        assert!(!stream.has_error());
        
        // Cause error
        let _ = stream.get_byte();
        assert!(stream.has_error());
        assert!(stream.error().is_some());
        
        // Clear error
        stream.clear_error();
        assert!(!stream.has_error());
        assert!(stream.error().is_none());
    }
}