use crate::core::date::{extract_game_date, scid_get_event_date};
use crate::core::error::{Result, ScidError};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ScidHeader {
    pub magic: [u8; 8],
    pub version: u16,
    pub base_type: u32,
    pub num_games: u32,
    pub auto_load: u32,
    pub description: String,
    pub custom_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GameFlags {
    pub start: bool,
    pub promotions: bool,
    pub under_promotions: bool,
    pub delete: bool,
    pub white_opening: bool,
    pub black_opening: bool,
    pub middlegame: bool,
    pub endgame: bool,
    pub novelty: bool,
    pub pawn_structure: bool,
    pub tactics: bool,
    pub kingside: bool,
    pub queenside: bool,
    pub brilliancy: bool,
    pub blunder: bool,
    pub user: bool,
}

#[derive(Debug, Clone)]
pub struct GameIndex {
    pub offset: u32,
    pub length: u32,
    pub white_id: u32,
    pub black_id: u32,
    pub event_id: u32,
    pub site_id: u32,
    pub round_id: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub event_year: Option<u16>,
    pub event_month: Option<u8>,
    pub event_day: Option<u8>,
    pub result: u8,
    pub eco: u16,
    pub white_elo: u16,
    pub black_elo: u16,
    pub flags: u16,
    pub parsed_flags: GameFlags,
    pub num_half_moves: u16,
}

pub struct Si4File {
    mmap: Mmap,
    header: ScidHeader,
}

impl Si4File {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| ScidError::FileOpen(e, path.to_path_buf()))?;
        let mmap =
            unsafe { Mmap::map(&file) }.map_err(|e| ScidError::Mmap(e, path.to_path_buf()))?;
        
        let header = Self::parse_header(&mmap)?;
        
        Ok(Self { mmap, header })
    }
    
    fn parse_header(data: &[u8]) -> Result<ScidHeader> {
        if data.len() < 182 {
            return Err(ScidError::invalid_format("SI4 file too small for header"));
        }
        
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&data[0..8]);
        
        let expected_magic = b"Scid.si\0";
        if magic != *expected_magic {
            return Err(ScidError::invalid_format(
                format!("Invalid magic header: expected {:?}, got {:?}", expected_magic, magic)
            ));
        }
        
        let version = u16::from_be_bytes([data[8], data[9]]);
        let base_type = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);
        
        let num_games = ((data[14] as u32) << 16) | ((data[15] as u32) << 8) | (data[16] as u32);
        let auto_load = ((data[17] as u32) << 16) | ((data[18] as u32) << 8) | (data[19] as u32);
        
        let description = Self::read_null_terminated_string(&data[20..128]);
        
        let mut custom_flags = Vec::new();
        for i in 0..6 {
            let start = 128 + i * 9;
            let flag_desc = Self::read_null_terminated_string(&data[start..start + 9]);
            custom_flags.push(flag_desc);
        }
        
        Ok(ScidHeader {
            magic,
            version,
            base_type,
            num_games,
            auto_load,
            description,
            custom_flags,
        })
    }
    
    fn read_null_terminated_string(data: &[u8]) -> String {
        if let Some(null_pos) = data.iter().position(|&b| b == 0) {
            String::from_utf8_lossy(&data[..null_pos]).to_string()
        } else {
            String::from_utf8_lossy(data).to_string()
        }
    }
    
    pub fn header(&self) -> &ScidHeader {
        &self.header
    }
    
    pub fn num_games(&self) -> u32 {
        self.header.num_games
    }
    
    pub fn games(&self) -> GameIterator {
        GameIterator {
            si4: self,
            current_index: 0,
        }
    }
    
    pub fn get_game(&self, index: u32) -> Result<GameIndex> {
        if index >= self.header.num_games {
            return Err(ScidError::game_index_out_of_bounds(index as usize, self.header.num_games as usize));
        }
        
        let entry_start = 182 + index as usize * 47;
        if entry_start + 47 > self.mmap.len() {
            return Err(ScidError::invalid_format("Game index entry beyond file bounds"));
        }
        
        Self::parse_game_index_entry(&self.mmap[entry_start..entry_start + 47])
    }
    
    fn parse_game_index_entry(data: &[u8]) -> Result<GameIndex> {
        let offset = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        
        let length_low = u16::from_be_bytes([data[4], data[5]]);
        let length_high = data[6];
        let length = Self::parse_game_length(length_low, length_high);
        
        let flags = u16::from_be_bytes([data[7], data[8]]);
        let parsed_flags = Self::parse_game_flags(flags);
        
        let white_black_high = data[9];
        let white_id_low = u16::from_be_bytes([data[10], data[11]]);
        let black_id_low = u16::from_be_bytes([data[12], data[13]]);
        let (white_id, black_id) = Self::parse_player_ids(white_black_high, white_id_low, black_id_low);
        
        let event_site_rnd_high = data[14];
        let event_id_low = u16::from_be_bytes([data[15], data[16]]);
        let site_id_low = u16::from_be_bytes([data[17], data[18]]);
        let round_id_low = u16::from_be_bytes([data[19], data[20]]);
        let (event_id, site_id, round_id) = Self::parse_event_site_round_ids(
            event_site_rnd_high, event_id_low, site_id_low, round_id_low
        );
        
        let var_counts = u16::from_be_bytes([data[21], data[22]]);
        let eco = u16::from_be_bytes([data[23], data[24]]);
        
        let dates_field = u32::from_be_bytes([data[25], data[26], data[27], data[28]]);
        let (year, month, day) = extract_game_date(dates_field);
        let (event_year, event_month, event_day) = scid_get_event_date(dates_field)
            .map(|(y, m, d)| (y as u16, m as u8, d as u8))
            .unwrap_or((0, 0, 0))
            .into();
        
        let white_elo_raw = u16::from_be_bytes([data[29], data[30]]);
        let black_elo_raw = u16::from_be_bytes([data[31], data[32]]);
        let white_elo = white_elo_raw & 0x0FFF;
        let black_elo = black_elo_raw & 0x0FFF;
        
        let num_half_moves_low = data[37];
        let num_half_moves = num_half_moves_low as u16 | (((data[38] >> 6) as u16) << 8);
        
        let result = (var_counts >> 12) as u8;
        
        Ok(GameIndex {
            offset,
            length,
            white_id,
            black_id,
            event_id,
            site_id,
            round_id,
            year: year as u16,
            month: month as u8,
            day: day as u8,
            event_year: if event_year > 0 { Some(event_year) } else { None },
            event_month: if event_month > 0 { Some(event_month) } else { None },
            event_day: if event_day > 0 { Some(event_day) } else { None },
            result,
            eco,
            white_elo,
            black_elo,
            flags,
            parsed_flags,
            num_half_moves,
        })
    }
    
    fn parse_game_length(length_low: u16, length_high: u8) -> u32 {
        let base_length = length_low as u32;
        let extended_bit = ((length_high as u32) & 0x80) << 9;
        base_length + extended_bit
    }
    
    fn parse_game_flags(flags: u16) -> GameFlags {
        GameFlags {
            start: (flags & (1 << 0)) != 0,
            promotions: (flags & (1 << 1)) != 0,
            under_promotions: (flags & (1 << 2)) != 0,
            delete: (flags & (1 << 3)) != 0,
            white_opening: (flags & (1 << 4)) != 0,
            black_opening: (flags & (1 << 5)) != 0,
            middlegame: (flags & (1 << 6)) != 0,
            endgame: (flags & (1 << 7)) != 0,
            novelty: (flags & (1 << 8)) != 0,
            pawn_structure: (flags & (1 << 9)) != 0,
            tactics: (flags & (1 << 10)) != 0,
            kingside: (flags & (1 << 11)) != 0,
            queenside: (flags & (1 << 12)) != 0,
            brilliancy: (flags & (1 << 13)) != 0,
            blunder: (flags & (1 << 14)) != 0,
            user: (flags & (1 << 15)) != 0,
        }
    }
    
    fn parse_player_ids(white_black_high: u8, white_id_low: u16, black_id_low: u16) -> (u32, u32) {
        let white_high = (white_black_high >> 4) as u32;
        let white_id = (white_high << 16) | (white_id_low as u32);
        
        let black_high = (white_black_high & 0xF) as u32;
        let black_id = (black_high << 16) | (black_id_low as u32);
        
        (white_id, black_id)
    }
    
    fn parse_event_site_round_ids(
        event_site_rnd_high: u8,
        event_id_low: u16,
        site_id_low: u16,
        round_id_low: u16,
    ) -> (u32, u32, u32) {
        let event_high = (event_site_rnd_high >> 5) as u32;
        let event_id = (event_high << 16) | (event_id_low as u32);
        
        let site_high = ((event_site_rnd_high >> 2) & 0x7) as u32;
        let site_id = (site_high << 16) | (site_id_low as u32);
        
        let round_high = (event_site_rnd_high & 0x3) as u32;
        let round_id = (round_high << 16) | (round_id_low as u32);
        
        (event_id, site_id, round_id)
    }
}

pub struct GameIterator<'a> {
    si4: &'a Si4File,
    current_index: u32,
}

impl<'a> Iterator for GameIterator<'a> {
    type Item = Result<GameIndex>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.si4.header().num_games {
            return None;
        }
        
        let result = self.si4.get_game(self.current_index);
        self.current_index += 1;
        Some(result)
    }
}

pub fn decode_result(result: u8) -> &'static str {
    match result {
        0 => "*",
        1 => "1-0",
        2 => "0-1",
        3 => "1/2-1/2",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_length_parsing() {
        assert_eq!(Si4File::parse_game_length(0x1234, 0x00), 0x1234);
        assert_eq!(Si4File::parse_game_length(0x1234, 0x80), 0x11234);
    }

    #[test]
    fn test_event_site_round_parsing() {
        let (event, site, round) = Si4File::parse_event_site_round_ids(0x65, 0x1234, 0x5678, 0x9ABC);
        assert_eq!(event, 0x31234);
        assert_eq!(site, 0x15678);
        assert_eq!(round, 0x19ABC);
    }

    #[test]
    fn test_flag_parsing() {
        let flags = Si4File::parse_game_flags(0xFFFF);
        assert!(flags.start);
        assert!(flags.promotions);
        assert!(flags.user);
        
        let flags = Si4File::parse_game_flags(0x0000);
        assert!(!flags.start);
        assert!(!flags.promotions);
        assert!(!flags.user);
    }

    #[test]
    fn test_result_decoding() {
        assert_eq!(decode_result(0), "*");
        assert_eq!(decode_result(1), "1-0");
        assert_eq!(decode_result(2), "0-1");
        assert_eq!(decode_result(3), "1/2-1/2");
    }

    #[test]
    fn test_header_parsing() {
        let test_data = create_test_header();
        let header = Si4File::parse_header(&test_data).unwrap();
        
        assert_eq!(&header.magic, b"Scid.si\0");
        assert_eq!(header.version, 400);
        assert_eq!(header.num_games, 5);
        assert_eq!(header.description, "Test");
    }

    fn create_test_header() -> Vec<u8> {
        let mut data = vec![0u8; 182];
        
        data[0..8].copy_from_slice(b"Scid.si\0");
        data[8..10].copy_from_slice(&400u16.to_be_bytes());
        data[10..14].copy_from_slice(&0u32.to_be_bytes());
        data[14] = 0;
        data[15] = 0;
        data[16] = 5;
        data[17] = 0;
        data[18] = 0;
        data[19] = 2;
        data[20..24].copy_from_slice(b"Test");
        
        data
    }
}
