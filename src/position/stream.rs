
pub struct ScidByteStream<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ScidByteStream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn get_consumed_bytes(&self, start: usize) -> &[u8] {
        &self.bytes[start..self.position]
    }
}
