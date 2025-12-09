/// Unified Piece Identification System
/// 
/// This module implements Phase 4.1: Eliminate Redundant Systems
/// The goal is to move toward unified piece identification to eliminate
/// dual numbering system conflicts.
/// 
/// **Problem**: SCID uses two different piece numbering systems:
/// 1. Move Encoding: piece_num 1-6 maps to piece types
/// 2. Board Lists: indices 0-11 map to piece locations
/// 
/// **Solution**: UnifiedPiece type that maintains both piece type and index
/// in a single, consistent structure.

use shakmaty::Role;

/// Unified piece identification that eliminates dual numbering conflicts
/// 
/// This structure maintains the actual piece type and its board index
/// in a single place, preventing inconsistencies between move encoding
/// and board tracking systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnifiedPiece {
    /// Actual piece type (King, Queen, Rook, Bishop, Knight, Pawn)
    pub piece_type: PieceType,
    /// Board list index (0-11) - location in piece_lists array
    pub board_index: u8,
    /// Move encoding number (1-6) - for SCID move compatibility
    pub move_encoding_num: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    Empty,
}

impl UnifiedPiece {
    /// Create a unified piece from move encoding (SG4 interpretation)
    /// 
    /// Maps move encoding numbers (1-6) to unified representation
    pub fn from_move_encoding(piece_num: u8, board_index: u8) -> Option<Self> {
        let piece_type = match piece_num {
            1 => PieceType::King,
            2 => PieceType::Queen,
            3 => PieceType::Rook,
            4 => PieceType::Bishop,
            5 => PieceType::Knight,
            6 => PieceType::Pawn,
            _ => return None,
        };
        
        Some(UnifiedPiece {
            piece_type,
            board_index,
            move_encoding_num: piece_num,
        })
    }
    
    /// Create a unified piece from shakmaty role
    /// 
    /// Maps shakmaty Role to unified representation
    pub fn from_role(role: Role, board_index: u8) -> Self {
        let (piece_type, move_encoding_num) = match role {
            Role::King => (PieceType::King, 1),
            Role::Queen => (PieceType::Queen, 2),
            Role::Rook => (PieceType::Rook, 3),
            Role::Bishop => (PieceType::Bishop, 4),
            Role::Knight => (PieceType::Knight, 5),
            Role::Pawn => (PieceType::Pawn, 6),
        };
        
        UnifiedPiece {
            piece_type,
            board_index,
            move_encoding_num,
        }
    }
    
    /// Get move encoding number for SCID compatibility
    /// 
    /// This provides the correct piece_num for move encoding,
    /// eliminating dual numbering conflicts.
    pub fn move_encoding_num(&self) -> u8 {
        self.move_encoding_num
    }
    
    /// Get board index for piece list access
    /// 
    /// This provides the correct index for board tracking,
    /// eliminating index/piece_type mismatches.
    pub fn board_index(&self) -> u8 {
        self.board_index
    }
    
    /// Get piece type for move routing
    /// 
    /// This provides consistent piece type for decoder routing,
    /// eliminating piece_num/interpretation conflicts.
    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }
    
    /// Check if this is a promotion-capable pawn
    /// 
    /// Used for enhanced move validation
    pub fn is_promotable_pawn(&self) -> bool {
        self.piece_type == PieceType::Pawn
    }
    
    /// Check if this is a sliding piece (Queen, Rook, Bishop)
    /// 
    /// Used for move validation optimizations
    pub fn is_sliding_piece(&self) -> bool {
        matches!(self.piece_type, PieceType::Queen | PieceType::Rook | PieceType::Bishop)
    }
}

/// Compatibility layer for migration from dual to unified system
/// 
/// This provides bridge functions to gradually eliminate redundant systems
pub struct UnifiedPieceCompatibility;

impl UnifiedPieceCompatibility {
    /// Convert legacy ScidMove to unified representation
    /// 
    /// This is the migration bridge from dual numbering to unified system
    pub fn from_legacy_scid_move(scid_move: &crate::position::ScidMove) -> UnifiedPiece {
        // Extract piece type from ScidMove (this should be correct after Phase 1-3 fixes)
        let piece_type = match scid_move.moving_piece {
            crate::position::PieceType::King => PieceType::King,
            crate::position::PieceType::Queen => PieceType::Queen,
            crate::position::PieceType::Rook => PieceType::Rook,
            crate::position::PieceType::Bishop => PieceType::Bishop,
            crate::position::PieceType::Knight => PieceType::Knight,
            crate::position::PieceType::Pawn => PieceType::Pawn,
            crate::position::PieceType::Empty => PieceType::Empty,
        };
        
        let move_encoding_num = match piece_type {
            PieceType::King => 1,
            PieceType::Queen => 2,
            PieceType::Rook => 3,
            PieceType::Bishop => 4,
            PieceType::Knight => 5,
            PieceType::Pawn => 6,
            PieceType::Empty => 0,
        };
        
        UnifiedPiece {
            piece_type,
            board_index: scid_move.piece_num, // Legacy board index
            move_encoding_num,
        }
    }
    
    /// Convert unified piece back to legacy piece_num for compatibility
    /// 
    /// This maintains backward compatibility during migration
    pub fn to_legacy_piece_num(unified: &UnifiedPiece) -> u8 {
        unified.board_index // Prefer board index for legacy compatibility
    }
    
    /// Convert unified piece back to legacy piece_type for compatibility
    /// 
    /// This maintains backward compatibility during migration
    pub fn to_legacy_piece_type(unified: &UnifiedPiece) -> crate::position::PieceType {
        match unified.piece_type {
            PieceType::King => crate::position::PieceType::King,
            PieceType::Queen => crate::position::PieceType::Queen,
            PieceType::Rook => crate::position::PieceType::Rook,
            PieceType::Bishop => crate::position::PieceType::Bishop,
            PieceType::Knight => crate::position::PieceType::Knight,
            PieceType::Pawn => crate::position::PieceType::Pawn,
            PieceType::Empty => crate::position::PieceType::Empty,
        }
    }
}