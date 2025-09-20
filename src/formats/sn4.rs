use crate::core::error::{Result, ScidError};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Sn4Header {
    pub magic: [u8; 8],
    pub timestamp: u32,
    pub num_names_player: u32,
    pub num_names_event: u32,
    pub num_names_site: u32,
    pub num_names_round: u32,
    pub max_frequency_player: u32,
    pub max_frequency_event: u32,
    pub max_frequency_site: u32,
    pub max_frequency_round: u32,
}

#[derive(Debug, Clone)]
pub struct NameRecord {
    pub id: u32,
    pub frequency: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum NameType {
    Player,
    Event,
    Site,
    Round,
}

pub struct Sn4File {
    mmap: Mmap,
    header: Sn4Header,
    player_offset: usize,
    event_offset: usize,
    site_offset: usize,
    round_offset: usize,
}

impl Sn4File {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| ScidError::FileOpen(e, path.to_path_buf()))?;
        let mmap =
            unsafe { Mmap::map(&file) }.map_err(|e| ScidError::Mmap(e, path.to_path_buf()))?;
        
        let header = Self::parse_header(&mmap)?;
        
        let player_offset = 36;
        let event_offset = player_offset + Self::calculate_section_size(&mmap, player_offset, header.num_names_player, header.max_frequency_player)?;
        let site_offset = event_offset + Self::calculate_section_size(&mmap, event_offset, header.num_names_event, header.max_frequency_event)?;
        let round_offset = site_offset + Self::calculate_section_size(&mmap, site_offset, header.num_names_site, header.max_frequency_site)?;
        
        Ok(Self {
            mmap,
            header,
            player_offset,
            event_offset,
            site_offset,
            round_offset,
        })
    }
    
    fn parse_header(data: &[u8]) -> Result<Sn4Header> {
        if data.len() < 36 {
            return Err(ScidError::invalid_format("SN4 file too small for header"));
        }
        
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&data[0..8]);
        
        let expected_magic = b"Scid.sn\0";
        if magic != *expected_magic {
            return Err(ScidError::invalid_format(
                format!("Invalid magic header: expected {:?}, got {:?}", expected_magic, magic)
            ));
        }
        
        let timestamp = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        
        let num_names_player = Self::read_u24_be(&data[12..15]);
        let num_names_event = Self::read_u24_be(&data[15..18]);
        let num_names_site = Self::read_u24_be(&data[18..21]);
        let num_names_round = Self::read_u24_be(&data[21..24]);
        
        let max_frequency_player = Self::read_u24_be(&data[24..27]);
        let max_frequency_event = Self::read_u24_be(&data[27..30]);
        let max_frequency_site = Self::read_u24_be(&data[30..33]);
        let max_frequency_round = Self::read_u24_be(&data[33..36]);
        
        Ok(Sn4Header {
            magic,
            timestamp,
            num_names_player,
            num_names_event,
            num_names_site,
            num_names_round,
            max_frequency_player,
            max_frequency_event,
            max_frequency_site,
            max_frequency_round,
        })
    }
    
    fn read_u24_be(data: &[u8]) -> u32 {
        ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32)
    }
    
    fn calculate_section_size(data: &[u8], offset: usize, num_names: u32, max_frequency: u32) -> Result<usize> {
        if num_names == 0 {
            return Ok(0);
        }
        
        let mut current_offset = offset;
        let mut previous_name = String::new();
        
        for i in 0..num_names {
            let record = Self::parse_name_record_at_offset(data, &mut current_offset, i, num_names, max_frequency, &previous_name)?;
            previous_name = record.name;
        }
        
        Ok(current_offset - offset)
    }
    
    fn parse_name_record_at_offset(
        data: &[u8],
        offset: &mut usize,
        record_index: u32,
        num_names: u32,
        max_frequency: u32,
        previous_name: &str,
    ) -> Result<NameRecord> {
        let id = if num_names >= 65536 {
            Self::read_u24_be_at(data, *offset)
        } else {
            Self::read_u16_be_at(data, *offset) as u32
        };
        *offset += if num_names >= 65536 { 3 } else { 2 };
        
        let frequency = if max_frequency >= 65536 {
            Self::read_u24_be_at(data, *offset)
        } else if max_frequency >= 256 {
            Self::read_u16_be_at(data, *offset) as u32
        } else {
            data[*offset] as u32
        };
        *offset += if max_frequency >= 65536 { 3 } else if max_frequency >= 256 { 2 } else { 1 };
        
        let total_length = data[*offset] as usize;
        *offset += 1;
        
        let prefix_length = if record_index > 0 {
            data[*offset] as usize
        } else {
            0
        };
        *offset += 1;
        
        if prefix_length > total_length {
            return Err(ScidError::invalid_format(
                format!("Invalid prefix length {} > total length {}", prefix_length, total_length)
            ));
        }
        
        let suffix_length = total_length - prefix_length;
        
        if *offset + suffix_length > data.len() {
            return Err(ScidError::invalid_format("Name record extends beyond file bounds"));
        }
        
        let mut name_bytes = Vec::with_capacity(total_length);
        
        if prefix_length > 0 {
            let previous_bytes = previous_name.as_bytes();
            if prefix_length > previous_bytes.len() {
                return Err(ScidError::invalid_format(
                    format!("Prefix length {} exceeds previous name length {}", 
                        prefix_length, previous_bytes.len())
                ));
            }
            name_bytes.extend_from_slice(&previous_bytes[..prefix_length]);
        }
        
        name_bytes.extend_from_slice(&data[*offset..*offset + suffix_length]);
        *offset += suffix_length;
        
        let name = String::from_utf8_lossy(&name_bytes)
            .trim_end_matches('\0')
            .chars()
            .filter(|&c| c >= ' ' || c == '\t' || c == '\n')
            .collect::<String>()
            .trim()
            .to_string();
        
        Ok(NameRecord {
            id,
            frequency,
            name,
        })
    }
    
    fn read_u16_be_at(data: &[u8], offset: usize) -> u16 {
        u16::from_be_bytes([data[offset], data[offset + 1]])
    }
    
    fn read_u24_be_at(data: &[u8], offset: usize) -> u32 {
        ((data[offset] as u32) << 16) | ((data[offset + 1] as u32) << 8) | (data[offset + 2] as u32)
    }
    
    pub fn header(&self) -> &Sn4Header {
        &self.header
    }
    
    pub fn get_name(&self, name_type: NameType, id: u32) -> Result<Option<String>> {
        let (offset, num_names, max_frequency) = match name_type {
            NameType::Player => (self.player_offset, self.header.num_names_player, self.header.max_frequency_player),
            NameType::Event => (self.event_offset, self.header.num_names_event, self.header.max_frequency_event),
            NameType::Site => (self.site_offset, self.header.num_names_site, self.header.max_frequency_site),
            NameType::Round => (self.round_offset, self.header.num_names_round, self.header.max_frequency_round),
        };
        
        if id >= num_names {
            return Ok(None);
        }
        
        let mut current_offset = offset;
        let mut previous_name = String::new();
        
        for i in 0..=id {
            let record = Self::parse_name_record_at_offset(&self.mmap, &mut current_offset, i, num_names, max_frequency, &previous_name)?;
            if i == id {
                return Ok(Some(record.name));
            }
            previous_name = record.name;
        }
        
        Ok(None)
    }
    
    pub fn iter_names(&self, name_type: NameType) -> NameIterator {
        let (offset, num_names, max_frequency) = match name_type {
            NameType::Player => (self.player_offset, self.header.num_names_player, self.header.max_frequency_player),
            NameType::Event => (self.event_offset, self.header.num_names_event, self.header.max_frequency_event),
            NameType::Site => (self.site_offset, self.header.num_names_site, self.header.max_frequency_site),
            NameType::Round => (self.round_offset, self.header.num_names_round, self.header.max_frequency_round),
        };
        
        NameIterator {
            sn4: self,
            current_offset: offset,
            current_index: 0,
            num_names,
            max_frequency,
            previous_name: String::new(),
        }
    }
}

pub struct NameIterator<'a> {
    sn4: &'a Sn4File,
    current_offset: usize,
    current_index: u32,
    num_names: u32,
    max_frequency: u32,
    previous_name: String,
}

impl<'a> Iterator for NameIterator<'a> {
    type Item = Result<NameRecord>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.num_names {
            return None;
        }
        
        let result = Sn4File::parse_name_record_at_offset(
            &self.sn4.mmap,
            &mut self.current_offset,
            self.current_index,
            self.num_names,
            self.max_frequency,
            &self.previous_name,
        );
        
        if let Ok(ref record) = result {
            self.previous_name = record.name.clone();
        }
        
        self.current_index += 1;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parsing() {
        let test_data = create_test_header();
        let header = Sn4File::parse_header(&test_data).unwrap();
        
        assert_eq!(&header.magic, b"Scid.sn\0");
        assert_eq!(header.timestamp, 1234567890);
        assert_eq!(header.num_names_player, 256000);
        assert_eq!(header.num_names_event, 500);
        assert_eq!(header.num_names_site, 200);
        assert_eq!(header.num_names_round, 100);
    }

    #[test]
    fn test_u24_parsing() {
        assert_eq!(Sn4File::read_u24_be(&[0x12, 0x34, 0x56]), 0x123456);
        assert_eq!(Sn4File::read_u24_be(&[0xFF, 0xFF, 0xFF]), 0xFFFFFF);
    }

    fn create_test_header() -> Vec<u8> {
        let mut data = vec![0u8; 36];
        
        data[0..8].copy_from_slice(b"Scid.sn\0");
        data[8..12].copy_from_slice(&1234567890u32.to_be_bytes());
        data[12] = 3;
        data[13] = 232; // 1000 in 3-byte BE (3 * 256 + 232 = 1000)
        data[15] = 0;
        data[16] = 1;
        data[17] = 244; // 500 in 3-byte BE
        data[18] = 0;
        data[19] = 0;
        data[20] = 200; // 200 in 3-byte BE
        data[21] = 0;
        data[22] = 0;
        data[23] = 100; // 100 in 3-byte BE
        data[24] = 0;
        data[25] = 0;
        data[26] = 10; // 10 in 3-byte BE
        data[27] = 0;
        data[28] = 0;
        data[29] = 5; // 5 in 3-byte BE
        data[30] = 0;
        data[31] = 0;
        data[32] = 15; // 15 in 3-byte BE
        data[33] = 0;
        data[34] = 0;
        data[35] = 8; // 8 in 3-byte BE
        
        data
    }
}
