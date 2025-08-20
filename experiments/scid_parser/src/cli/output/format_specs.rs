// Format specification displays

/// Display comprehensive SCID database format specifications
pub fn display_scid_format_specifications() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                          SCID DATABASE FORMAT SPECIFICATIONS");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // Display SI4 format
    display_si4_format_specification();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // Display SN4 format  
    display_sn4_format_specification();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // Display SG4 format
    display_sg4_format_specification();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                                 IMPLEMENTATION NOTES");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("• All multi-byte integers use BIG-ENDIAN byte order");
    println!("• SCID uses proprietary binary encoding throughout");
    println!("• This implementation reverse-engineered from scidvspc source code");
    println!("• Date encoding: ((year << 9) | (month << 5) | day) with no year offset");
    println!("• Name compression: Front-coded strings with variable-length IDs/frequencies");
    println!("• Move encoding: 1-3 bytes per move depending on piece type and complexity");
}

/// Display SI4 (Index) format specification
fn display_si4_format_specification() {
    println!("📁 SI4 INDEX FILE FORMAT (.si4)");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!();
    
    println!("HEADER STRUCTURE (182 bytes):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Offset │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│   0-7  │ 8 bytes  │ ASCII   │ Magic: \"Scid.si\\0\"                              │");
    println!("│   8-9  │ 2 bytes  │ BE uint │ Version (usually 400)                           │");
    println!("│  10-13 │ 4 bytes  │ BE uint │ Base Type                                       │");
    println!("│  14-16 │ 3 bytes  │ BE uint │ Number of Games                                 │");
    println!("│  17-19 │ 3 bytes  │ BE uint │ Auto Load Game                                  │");
    println!("│  20-127│108 bytes │ String  │ Description (null-terminated)                   │");
    println!("│128-181 │ 54 bytes │ Strings │ Custom Flag Descriptions (6 × 9 bytes each)    │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
    
    println!("GAME INDEX ENTRIES (47 bytes each):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Offset │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│   0-3  │ 4 bytes  │ BE uint │ Game File Offset                                │");
    println!("│   4-5  │ 2 bytes  │ BE uint │ Game Length (low 16 bits)                       │");
    println!("│    6   │ 1 byte   │ uint8   │ Game Length (high 1 bit) + flags               │");
    println!("│   7-8  │ 2 bytes  │ BE uint │ Game Flags (16 types)                          │");
    println!("│    9   │ 1 byte   │ packed  │ Player ID high bits (4+4)                       │");
    println!("│  10-11 │ 2 bytes  │ BE uint │ White Player ID (low 16 bits)                   │");
    println!("│  12-13 │ 2 bytes  │ BE uint │ Black Player ID (low 16 bits)                   │");
    println!("│   14   │ 1 byte   │ packed  │ Event/Site/Round ID high bits (3+3+2)          │");
    println!("│  15-16 │ 2 bytes  │ BE uint │ Event ID (low 16 bits)                          │");
    println!("│  17-18 │ 2 bytes  │ BE uint │ Site ID (low 16 bits)                           │");
    println!("│  19-20 │ 2 bytes  │ BE uint │ Round ID (low 16 bits)                          │");
    println!("│  21-22 │ 2 bytes  │ BE uint │ Variation Counts + Result (top 4 bits)          │");
    println!("│  23-24 │ 2 bytes  │ BE uint │ ECO Code                                        │");
    println!("│  25-28 │ 4 bytes  │ BE uint │ Game/Event Dates (packed format)               │");
    println!("│  29-30 │ 2 bytes  │ BE uint │ White ELO (12 bits) + Rating Type (4 bits)     │");
    println!("│  31-32 │ 2 bytes  │ BE uint │ Black ELO (12 bits) + Rating Type (4 bits)     │");
    println!("│  33-36 │ 4 bytes  │ BE uint │ Material Signature (final position)            │");
    println!("│   37   │ 1 byte   │ uint8   │ Half Moves (low 8 bits)                         │");
    println!("│  38-46 │ 9 bytes  │ packed  │ Pawn Data + Half Moves high bits                │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
}

/// Display SN4 (Names) format specification  
fn display_sn4_format_specification() {
    println!("📂 SN4 NAME FILE FORMAT (.sn4)");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!();
    
    println!("HEADER STRUCTURE (36 bytes):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Offset │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│   0-7  │ 8 bytes  │ ASCII   │ Magic: \"Scid.sn\\0\"                              │");
    println!("│   8-11 │ 4 bytes  │ BE uint │ Timestamp                                       │");
    println!("│  12-14 │ 3 bytes  │ BE uint │ Number of Player Names                          │");
    println!("│  15-17 │ 3 bytes  │ BE uint │ Number of Event Names                           │");
    println!("│  18-20 │ 3 bytes  │ BE uint │ Number of Site Names                            │");
    println!("│  21-23 │ 3 bytes  │ BE uint │ Number of Round Names                           │");
    println!("│  24-26 │ 3 bytes  │ BE uint │ Max Frequency for Players                       │");
    println!("│  27-29 │ 3 bytes  │ BE uint │ Max Frequency for Events                        │");
    println!("│  30-32 │ 3 bytes  │ BE uint │ Max Frequency for Sites                         │");
    println!("│  33-35 │ 3 bytes  │ BE uint │ Max Frequency for Rounds                        │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
    
    println!("NAME RECORD STRUCTURE (Variable Length):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Field  │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│ Name ID│ 2-3 bytes│ BE uint │ Sequential ID (2 bytes if count<65536)          │");
    println!("│Frequency│1-3 bytes│ BE uint │ Usage frequency (1/2/3 bytes based on max)     │");
    println!("│ Length │ 1 byte   │ uint8   │ Name string length                              │");
    println!("│ Name   │ N bytes  │ String  │ Front-coded compressed name                     │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
    println!("• Names stored in 4 sections: Player, Event, Site, Round");
    println!("• Front-coding: Each name stores only the suffix after common prefix");
    println!("• Variable-length encoding optimizes for database size");
}

/// Display SG4 (Games) format specification
fn display_sg4_format_specification() {
    println!("🎮 SG4 GAME FILE FORMAT (.sg4)");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!();
    
    println!("FILE STRUCTURE:");
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ • Block-based organization: 131,072-byte blocks                            │");
    println!("│ • Variable-length game records (no fixed headers)                          │");
    println!("│ • Games separated by ENCODE_END_GAME (15) markers                          │");
    println!("│ • Complex games may span multiple blocks                                   │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("GAME RECORD STRUCTURE (Variable Length):");
    println!("┌─────────────┬─────────────────────────────────────────────────────────────────┐");
    println!("│ Component   │ Description                                                     │");
    println!("├─────────────┼─────────────────────────────────────────────────────────────────┤");
    println!("│ PGN Tags    │ Non-standard tags (WhiteTitle, BlackTitle, etc.)               │");
    println!("│ Game Flags  │ Promotion flags, non-standard starts                           │");
    println!("│ Move Data   │ 1-3 byte move encodings + annotations                          │");
    println!("│ Variations  │ Nested alternative move sequences                              │");
    println!("│ Comments    │ Null-terminated text strings                                   │");
    println!("│ NAGs        │ Numeric Annotation Glyphs (!, ?, !!, etc.)                    │");
    println!("│ End Marker  │ ENCODE_END_GAME (15) - marks game completion                   │");
    println!("└─────────────┴─────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("MOVE ENCODING (1-3 bytes per move):");
    println!("┌─────────┬───────────┬─────────────────────────────────────────────────────────┐");
    println!("│ Piece   │ Bytes     │ Encoding Method                                         │");
    println!("├─────────┼───────────┼─────────────────────────────────────────────────────────┤");
    println!("│ King    │ 1-2 bytes │ Direction/castling codes + complex scenarios           │");
    println!("│ Queen   │ 1-2 bytes │ Rook-like moves: 1 byte, Diagonal moves: 2 bytes      │");
    println!("│ Rook    │ 1 byte    │ Target rank/file encoded in 4 bits                     │");
    println!("│ Bishop  │ 1 byte    │ Target file + direction in 4 bits                      │");
    println!("│ Knight  │ 1 byte    │ L-shaped move pattern in 4 bits                        │");
    println!("│ Pawn    │ 1-2 bytes │ Direction + promotion, complex promotions: 2 bytes     │");
    println!("└─────────┴───────────┴─────────────────────────────────────────────────────────┘");
    println!();
    
    println!("SPECIAL ENCODING VALUES:");
    println!("┌───────┬─────────────────────────────────────────────────────────────────────────┐");
    println!("│ Value │ Meaning                                                                 │");
    println!("├───────┼─────────────────────────────────────────────────────────────────────────┤");
    println!("│  0-10 │ Regular moves (piece_num << 4 | move_value)                            │");
    println!("│   11  │ ENCODE_NAG - followed by NAG value byte                                │");
    println!("│   12  │ ENCODE_COMMENT - followed by null-terminated string                    │");
    println!("│   13  │ ENCODE_START_MARKER - begin variation                                  │");
    println!("│   14  │ ENCODE_END_MARKER - end variation                                      │");
    println!("│   15  │ ENCODE_END_GAME - end of game record                                   │");
    println!("└───────┴─────────────────────────────────────────────────────────────────────────┘");
}