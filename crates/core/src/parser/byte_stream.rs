use crate::error::{Result, ScidError};
use std::path::PathBuf;

/// Sequential byte stream reader compatible with SCID's ByteBuffer
///
/// Replicates functionality from scidvspc/src/bytebuf.h ByteBuffer class
/// Supports reading bytes sequentially with position tracking
#[derive(Debug)]
pub struct ByteStream<'a> {
    /// Underlying byte buffer
    buffer: &'a [u8],

    /// Current read position
    position: usize,

    /// Total bytes available
    length: usize,
}

impl<'a> ByteStream<'a> {
    /// Create new ByteStream from byte slice
    pub fn new(buffer: &'a [u8]) -> Self {
        ByteStream {
            buffer,
            position: 0,
            length: buffer.len(),
        }
    }

    /// Read next byte from stream and advance position
    /// Equivalent to SCID's ByteBuffer::GetByte()
    pub fn get_byte(&mut self) -> Result<u8> {
        if self.position >= self.length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("byte_stream"),
                offset: self.position as u64,
                message: "Buffer underrun: no more bytes available".to_string(),
            });
        }

        let byte = self.buffer[self.position];
        self.position += 1;
        Ok(byte)
    }

    /// Peek at next byte without advancing position
    pub fn peek_byte(&self) -> Result<u8> {
        if self.position >= self.length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("byte_stream"),
                offset: self.position as u64,
                message: "Buffer underrun: no more bytes to peek".to_string(),
            });
        }

        Ok(self.buffer[self.position])
    }

    /// Skip N bytes forward
    pub fn skip(&mut self, count: usize) -> Result<()> {
        if self.position + count > self.length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("byte_stream"),
                offset: self.position as u64,
                message: format!("Buffer underrun: cannot skip {} bytes", count),
            });
        }

        self.position += count;
        Ok(())
    }

    /// Get current position in stream
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes available
    pub fn remaining(&self) -> usize {
        self.length - self.position
    }

    /// Check if stream has more bytes
    pub fn has_more(&self) -> bool {
        self.position < self.length
    }

    /// Reset stream to beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_stream_basic() {
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let mut stream = ByteStream::new(&data);

        assert_eq!(stream.get_byte().unwrap(), 0x10);
        assert_eq!(stream.get_byte().unwrap(), 0x20);
        assert_eq!(stream.position(), 2);
        assert_eq!(stream.remaining(), 2);
    }

    #[test]
    fn test_byte_stream_peek() {
        let data = vec![0x10, 0x20, 0x30];
        let mut stream = ByteStream::new(&data);

        // Peek doesn't advance position
        assert_eq!(stream.peek_byte().unwrap(), 0x10);
        assert_eq!(stream.peek_byte().unwrap(), 0x10);
        assert_eq!(stream.position(), 0);

        // Get does advance
        assert_eq!(stream.get_byte().unwrap(), 0x10);
        assert_eq!(stream.position(), 1);
    }

    #[test]
    fn test_byte_stream_underrun() {
        let data = vec![0x10];
        let mut stream = ByteStream::new(&data);

        stream.get_byte().unwrap();

        // Should error on second byte
        let result = stream.get_byte();
        assert!(result.is_err());
    }

    #[test]
    fn test_byte_stream_skip() {
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let mut stream = ByteStream::new(&data);

        stream.skip(2).unwrap();
        assert_eq!(stream.get_byte().unwrap(), 0x30);
    }

    #[test]
    fn test_byte_stream_reset() {
        let data = vec![0x10, 0x20, 0x30];
        let mut stream = ByteStream::new(&data);

        stream.get_byte().unwrap();
        stream.get_byte().unwrap();

        stream.reset();
        assert_eq!(stream.position(), 0);
        assert_eq!(stream.get_byte().unwrap(), 0x10);
    }
}
