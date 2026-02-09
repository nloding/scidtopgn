# SCID Database Encoding/Decoding Source Files

Complete reference listing of all SCID source code files that handle database encoding and decoding operations for .si4 (index), .sn4 (name), and .sg4 (game) file formats.

## Core Common Definitions

### `scidvspc/src/common.h` (20,274 bytes)
- **Primary Responsibility**: Common macros, structures, and constants used throughout SCID
- **Key Definitions**:
  - SCID_VERSION constants (file format version 400 = 4.0)
  - Buffer size constants (BBUF_SIZE, TBUF_SIZE)
  - File suffix constants (.si4, .sn4, .sg4, .pgn, etc.)
  - Bit manipulation macros (BIT_0 through BIT_7)
  - Upper/lower bit extraction macros
  - Type definitions (versionT, etc.)
  - Zlib interface definitions
  - Error handling constants

## Core Index Files (.si4 encoding/decoding)

### `scidvspc/src/index.cpp` (51,851 bytes)
- **Primary Responsibility**: Index file (.si4) reading and writing operations
- **Key Functions**:
  - Index entry serialization/deserialization
  - Header parsing and validation
  - Game metadata encoding/decoding
  - Date field encoding (packed game + event dates)
  - Bit field extraction for player/event/site/round IDs
  - Result and rating encoding

### `scidvspc/src/index.h` (26,868 bytes)
- **Primary Responsibility**: Index structure and bit field definitions
- **Key Definitions**:
  - `IndexEntry` struct (47-byte game index entry)
  - Header structure definitions (182-byte header)
  - Packed field bit extraction macros
  - Field offset constants
  - Name ID extraction helpers
  - Date field bit manipulation

### `scidvspc/src/date.h` (2,541 bytes)
- **Primary Responsibility**: Date encoding constants and bit manipulation
- **Key Definitions**:
  - Date encoding constants (20-bit game date, 12-bit event date)
  - Bit field layout specifications
  - Date structure definitions
  - Year offset calculation constants
  - Date validation helpers

### `scidvspc/src/date.cpp` (4,252 bytes)
- **Primary Responsibility**: Date handling implementation
- **Key Functions**:
  - Date encoding to packed 32-bit format
  - Date decoding from packed format
  - Game date parsing (year/month/day extraction)
  - Event date decoding (with year offset calculation)
  - Date validation and formatting

## Error Handling and Utilities

### `scidvspc/src/error.h` (1,978 bytes)
- **Primary Responsibility**: Error code definitions for all SCID operations
- **Key Definitions**:
  - Error code types (errorT typedef)
  - General errors (ERROR, etc.)
  - File I/O errors (ERROR_FileOpen, ERROR_FileWrite, ERROR_BadMagic, etc.)
  - Memory and data errors (ERROR_MallocFailed, ERROR_CorruptData)
  - NameBase and Index errors (ERROR_NameBaseFull, ERROR_NameNotFound, etc.)
  - Position errors (ERROR_InvalidFEN, ERROR_InvalidMove, ERROR_PieceCount)
  - Game errors (ERROR_Game, ERROR_Decode, ERROR_VariationLimit, etc.)
  - PGN and Buffer errors

### `scidvspc/src/myassert.cpp` (913 bytes)
- **Primary Responsibility**: Assertion implementation for development
- **Key Functions**:
  - `_MyAssert()` - Assertion failure reporting

### `scidvspc/src/myassert.h` (1,466 bytes)
- **Primary Responsibility**: ASSERT macro definition
- **Key Definitions**:
  - `ASSERT()` macro for runtime assertions
  - Conditional compilation support (ASSERTIONS flag)
  - Optimized version (no-op) when assertions disabled

## File I/O and Multi-byte Operations

### `scidvspc/src/mfile.cpp` (8,796 bytes)
- **Primary Responsibility**: Low-level multi-byte value I/O
- **Key Functions**:
  - `ReadTwoBytes()` - Big-endian 16-bit reads
  - `ReadFourBytes()` - Big-endian 32-bit reads
  - `WriteTwoBytes()` - Big-endian 16-bit writes
  - `WriteFourBytes()` - Big-endian 32-bit writes
  - File handle management
  - Position seeking

### `scidvspc/src/mfile.h` (4,637 bytes)
- **Primary Responsibility**: Multi-byte file I/O declarations
- **Key Declarations**:
  - Read/write function prototypes
  - File handle type definitions
  - Error handling constants
  - Endianness specifications (confirmed big-endian)

### `scidvspc/src/bytebuf.cpp` (8,549 bytes)
- **Primary Responsibility**: ByteBuffer streaming implementation
- **Key Functions**:
  - `GetByte()` - Read single byte with position advancement
  - `PutByte()` - Write single byte with position advancement
  - `GetFixedString()` - Read fixed-length string
  - `GetTerminatedString()` - Read null-terminated string
  - `Skip()` - Advance buffer position
  - Buffer position management and bounds checking

### `scidvspc/src/bytebuf.h` (3,220 bytes)
- **Primary Responsibility**: ByteBuffer streaming interface
- **Key Definitions**:
  - `ByteBuffer` class definition
  - Buffer position tracking variables
  - Error state management
  - Status codes and constants
  - Stream positioning methods

### `scidvspc/src/bytepack.h` (13,228 bytes)
- **Primary Responsibility**: Byte packing/unpacking utilities
- **Key Functions**:
  - Variable-length integer encoding/decoding
  - Bit field packing
  - Compact data structure serialization
  - Space-efficient value representation

## Name Processing Files (.sn4 encoding/decoding)

### `scidvspc/src/namebase.cpp` (15,357 bytes)
- **Primary Responsibility**: Name file (.sn4) handling
- **Key Functions**:
  - Front-coding compression algorithm implementation
  - Name record parsing (ID, frequency, string length, string data)
  - Variable-length integer decoding (1-3 bytes based on max value)
  - Prefix/suffix handling for front-coding
  - Character set normalization
  - Name cleanup (control character removal, whitespace trimming)
  - Name lookup by ID

### `scidvspc/src/namebase.h` (10,951 bytes)
- **Primary Responsibility**: Name storage structure definitions
- **Key Definitions**:
  - Name record structure
  - Front-coding algorithm specifications
  - Variable-length encoding rules
  - Name section organization (players, events, sites, rounds)
  - Namebase class interface

## Game File Processing (.sg4 encoding/decoding)

### `scidvspc/src/game.cpp` (166,118 bytes)
- **Primary Responsibility**: Game parsing and move encoding (LARGEST file)
- **Key Functions**:
  - **Move Decoding** (piece-specific):
    - `decodeKing()` - King move decoding (lines 59111-59126)
    - `decodeKnight()` - Knight L-shaped move decoding (lines 59149-59160)
    - `decodePawn()` - Pawn move decoding including promotions (lines 59284-59348)
    - `decodeRook()` - Rook horizontal/vertical move decoding (lines 59183-59195)
    - `decodeBishop()` - Bishop diagonal move decoding (lines 59215-59230)
    - `decodeQueen()` - Queen move decoding (rook + bishop, 2-byte diagonals) (lines 59264-59282)
  - **Game Structure Parsing**:
    - `skipTags()` - Skip PGN tags until null terminator (lines 1650-1680)
    - `DecodeStart()` - Decode game start and flags (lines 2800-2850)
    - `DecodeMoves()` - Decode move sequence
  - **Annotations**:
    - `encodeComments()` - Encode comments to comment section
    - NAG (Numeric Annotation Glyph) handling
  - **Variations**:
    - Variation tree encoding/decoding
    - Nested variation structure handling
  - **Special Moves**:
    - Castling encoding
    - En passant encoding
    - Pawn promotion handling
    - Underpromotion support

### `scidvspc/src/game.h` (20,718 bytes)
- **Primary Responsibility**: Game structure and encoding constants
- **Key Definitions**:
  - Game record structure
  - Move encoding constants:
    - `ENCODE_NAG = 11` (0x0B) - Annotation marker
    - `ENCODE_COMMENT = 12` (0x0C) - Comment marker
    - `ENCODE_START_MARKER = 13` (0x0D) - Variation start
    - `ENCODE_END_MARKER = 14` (0x0E) - Variation end
    - `ENCODE_END_GAME = 15` (0x0F) - Game end marker
  - Piece number definitions
  - Move structure definitions
  - Annotation structures

### `scidvspc/src/position.cpp` (109,514 bytes)
- **Primary Responsibility**: Chess position management and validation
- **Key Functions**:
  - **Piece List Management**:
    - `StdStart()` - Initialize standard starting position piece numbers
    - `AddPiece()` - Add piece with automatic numbering (King always #0)
  - **Move Execution**:
    - `DoSimpleMove()` - Execute move with capture swap algorithm
    - Move validation and legality checking
  - **Capture Handling**:
    - Last piece swap into captured slot
    - Piece number renumbering after captures
  - **Special Moves**:
    - Castling move execution (updates King + Rook)
    - Promotion execution (preserves piece number, changes type)
  - **Position Queries**:
    - Piece location lookup
    - Square content queries
    - Material count tracking

### `scidvspc/src/position.h` (12,953 bytes)
- **Primary Responsibility**: Position class and piece tracking structures
- **Key Definitions**:
  - **Piece Tracking Data Structures**:
    - `List[2][16]` - Square of each piece by color and number
    - `ListPos[64]` - Reverse lookup from square to piece number
    - `Count[2]` - Number of active pieces per color
  - **Standard Starting Piece Numbers**:
    - King: 0, QR: 1, QN: 2, QB: 3, Q: 4, KB: 5, KN: 6, KR: 7
    - Pawns: 8-15 (a-h file order)
  - Position state variables
  - Board representation

### `scidvspc/src/gfile.cpp` (8,752 bytes)
- **Primary Responsibility**: Game file (.sg4) I/O and block management
- **Key Functions**:
  - `ReadGame()` - Read specific game by offset and length from SG4
  - Block-based file organization (131,072-byte blocks)
  - Game boundary detection
  - File position management
  - Block allocation and deallocation

### `scidvspc/src/gfile.h` (2,195 bytes)
- **Primary Responsibility**: Game file structures
- **Key Definitions**:
  - `IndexEntry` structure with `offset` and `length` fields
  - Block size constants (GF_BLOCKSIZE = 131,072)
  - GFile class interface
  - Game record structure

## Utility and Support Files

### `scidvspc/src/hash.h` (10,968 bytes)
- **Primary Responsibility**: Pre-generated Zobrist hash values
- **Key Definitions**:
  - `goodHashValues[768]` - 12 pieces × 64 squares of 32-bit hash values
  - Hash values with guaranteed bit distribution
  - Each value differs from every other by at least 10 bits
  - Used for position hashing in encoding/decoding

### `scidvspc/src/attacks.h` (8,870 bytes)
- **Primary Responsibility**: Pre-computed attack square arrays
- **Key Definitions**:
  - `knightAttacks[66][9]` - Knight move destinations per square
  - Other piece attack arrays (bishop, rook, queen, king)
  - Pre-computed for fast move validation
  - Used during move encoding/decoding validation

### `scidvspc/src/sqlist.h` (3,080 bytes)
- **Primary Responsibility**: SquareList class for managing square collections
- **Key Definitions**:
  - SquareList class with array of squares
  - MAX_SQUARELIST = 65 (64 squares + null)
  - Methods: Add(), Get(), Remove(), Contains(), Clear()
  - Used for tracking squares during move processing

### `scidvspc/src/sqset.h` (2,547 bytes)
- **Primary Responsibility**: SquareSet class for efficient square membership testing
- **Key Definitions**:
  - SquareSet class using bitmaps (2 uints = 64 bits)
  - Methods: Add(), Remove(), Contains(), Clear()
  - O(1) square membership test using bit operations
  - Used for move validation and position tracking

### `scidvspc/src/movelist.cpp` (5,431 bytes)
- **Primary Responsibility**: MoveList class implementation
- **Key Functions**:
  - MoveList management (add, remove, find, sort)
  - Legal move generation and storage
  - Move ordering and selection
  - Used during encoding/decoding for move operations

### `scidvspc/src/movelist.h` (4,325 bytes)
- **Primary Responsibility**: MoveList class definition
- **Key Definitions**:
  - `simpleMoveT` structure (move representation without full game data)
  - MoveList class with array of simpleMoveT
  - MAX_LEGAL_MOVES = 256
  - Inline methods for fast move list operations
  - `isNullMove()` function for null move detection

### `scidvspc/src/tokens.h` (3,267 bytes)
- **Primary Responsibility**: Token definitions for PGN scanning
- **Key Definitions**:
  - Token type definitions (tokenT typedef)
  - PGN token constants:
    - TOKEN_MoveNum, TOKEN_Ignore
    - TOKEN_Move_Pawn, TOKEN_Move_Promote, TOKEN_Move_Piece
    - TOKEN_Move_Castle_King, TOKEN_Move_Castle_Queen, TOKEN_Move_Null
  - Token predicate functions (TOKEN_isMove, TOKEN_isPawnMove)
  - Used in PGN parsing for encoding/decoding

### `scidvspc/src/tree.cpp` (6,322 bytes)
- **Primary Responsibility**: Tree management implementation
- **Key Functions**:
  - Opening tree generation from games
  - Move frequency tracking
  - ECO code assignment
  - Tree caching for display
  - Used for analysis of encoded games

### `scidvspc/src/tree.h` (2,249 bytes)
- **Primary Responsibility**: Tree structure definitions
- **Key Definitions**:
  - `treeNodeT` structure (move data, frequency, score, ECO)
  - Tree node management constants
  - MAX_TREE_NODES = 60
  - Used for opening book tree from encoded games

### `scidvspc/src/crosstab.cpp` (45,866 bytes)
- **Primary Responsibility**: Crosstable generation
- **Key Functions**:
  - Tournament crosstable generation
  - Swiss system support
  - Player pairing tracking
  - Results calculation

### `scidvspc/src/crosstab.h` (6,825 bytes)
- **Primary Responsibility**: Crosstable structures
- **Key Definitions**:
  - `clashT` structure (result, gameNum, opponent, round)
  - Crosstable sort modes
  - Output formats (plain, HTML, LaTeX)
  - CROSSTABLE_MaxPlayers, CROSSTABLE_MaxRounds

### `scidvspc/src/misc.cpp` (37,697 bytes)
- **Primary Responsibility**: Miscellaneous helper functions
- **Key Functions**:
  - Common encoding/decoding utilities
  - String manipulation helpers
  - Number formatting functions
  - Error handling routines

### `scidvspc/src/misc.h` (11,360 bytes)
- **Primary Responsibility**: Common definitions and constants
- **Key Definitions**:
  - Shared utility declarations
  - Common type definitions
  - Error codes and constants
  - Macro definitions

### `scidvspc/src/sqmove.h` (9,633 bytes)
- **Primary Responsibility**: Square and move encoding utilities
- **Key Functions**:
  - Square index calculations (A1=0, H8=63)
  - Move representation helpers
  - File/rank extraction functions
  - Square arithmetic utilities

### `scidvspc/src/optable.cpp` (74,634 bytes)
- **Primary Responsibility**: Opening table operations
- **Key Functions**:
  - Opening encoding/decoding
  - ECO code lookup and storage
  - Opening tree navigation
  - Opening move generation

### `scidvspc/src/optable.h` (9,183 bytes)
- **Primary Responsibility**: Opening table structures
- **Key Definitions**:
  - Opening table structures
  - Opening record definitions
  - ECO code mappings
  - Opening lookup interfaces

## Game Conversion Tools

### `scidvspc/scidmerge.cpp` (8,186 bytes)
- **Primary Responsibility**: SCID database merge utility
- **Key Functions**:
  - Merge multiple SCID databases into one
  - Handle name consolidation
  - Index merging
  - Game data concatenation
  - Uses core DB encoding functions to write merged output

### `scidvspc/src/pgnscid.cpp` (9,145 bytes)
- **Primary Responsibility**: PGN to SCID conversion tool
- **Key Functions**:
  - Parse PGN format games
  - Encode games to SCID binary format
  - Header/metadata extraction
  - Move notation to binary move encoding
  - Tag processing and encoding

### `scidvspc/src/scidt.cpp` (26,958 bytes)
- **Primary Responsibility**: SCID text/tool utility
- **Key Functions**:
  - Database processing operations
  - Batch encoding/decoding
  - Data transformation utilities
  - Format validation

### `scidvspc/src/scidlet.cpp` (35,451 bytes)
- **Primary Responsibility**: Lightweight SCID operations
- **Key Functions**:
  - Simplified game encoding/decoding
  - Streamlined game processing
  - Core functionality subset
  - Performance-critical operations

### `scidvspc/src/eco2epd.cpp` (7,805 bytes)
- **Primary Responsibility**: ECO file to EPD format conversion
- **Key Functions**:
  - Parse Scid ECO text files
  - Convert to EPD (Extended Position Description) format
  - Opening classification
  - Position encoding

## Specialized Encoding/Decoding

### `scidvspc/src/pgnparse.cpp` (43,254 bytes)
- **Primary Responsibility**: PGN format parsing
- **Key Functions**:
  - PGN tokenization
  - Move notation parsing
  - Tag parsing
  - Variation tree construction
  - Annotation extraction

### `scidvspc/src/pgnparse.h` (4,867 bytes)
- **Primary Responsibility**: PGN parser interface
- **Key Definitions**:
  - PGN parser structures
  - Token definitions
  - Parser state management
  - Error handling

### `scidvspc/src/recog.cpp` (47,305 bytes)
- **Primary Responsibility**: Opening recognition
- **Key Functions**:
  - ECO code recognition
  - Opening classification
  - Opening signature encoding
  - Opening name lookup

### `scidvspc/src/recog.h` (1,788 bytes)
- **Primary Responsibility**: Recognition structures
- **Key Definitions**:
  - Opening recognition data structures
  - ECO code constants
  - Recognition result definitions

### `scidvspc/src/nagtext.h` (13,754 bytes)
- **Primary Responsibility**: NAG (Numeric Annotation Glyph) to text conversion
- **Key Definitions**:
  - `evalNagsRegular[]` - NAG symbols to text strings
  - Standard NAG values (1-9: !!, ?, !?, ??, etc.)
  - Position evaluation NAGs ($10-$26)
  - Time/space advantage NAGs
  - Detailed NAG descriptions
  - Used for converting encoded NAGs to readable PGN annotations

### `scidvspc/src/naglatex.h` (4,706 bytes)
- **Primary Responsibility**: NAG to LaTeX conversion
- **Key Definitions**:
  - NAG symbols to LaTeX formatting
  - Used for generating LaTeX documents from games

### `scidvspc/src/stored.cpp` (12,842 bytes)
- **Primary Responsibility**: StoredLine management
- **Key Functions**:
  - Game line storage and retrieval
  - Stored line matching
  - Game index management
  - Used for caching frequently used game lines

### `scidvspc/src/stored.h` (1,388 bytes)
- **Primary Responsibility**: StoredLine class definition
- **Key Definitions**:
  - StoredLine class for game line storage
  - MAX_STORED_LINES = 256
  - `CanMatch()`, `GetText()`, `GetGame()` methods
  - Used for quick line lookup during analysis

## Character and String Handling

### `scidvspc/src/charsetdetector.cpp` (3,799 bytes)
- **Primary Responsibility**: Character set detection implementation
- **Key Functions**:
  - Character encoding detection (UTF-8, Latin-1, Windows, DOS)
  - Character set validation
  - Encoding detection from text content

### `scidvspc/src/charsetdetector.h` (4,217 bytes)
- **Primary Responsibility**: CharsetDetector class interface
- **Key Definitions**:
  - CharSet enumeration (ASCII, Latin1, Windoze, DOS, UTF8)
  - CharsetDetector class methods
  - `isUTF8()`, `isLatin1()`, `isWindows()`, `isDOS()`, `isASCII()`
  - `detect()`, `finish()`, `reset()`
  - Used for detecting encoding of player names and comments

### `scidvspc/src/charsetconverter.cpp` (38,062 bytes)
- **Primary Responsibility**: Character encoding conversion
- **Key Functions**:
  - Character set detection
  - UTF-8 encoding/decoding
  - Legacy encoding conversion
  - Text normalization

### `scidvspc/src/charsetconverter.h` (8,242 bytes)
- **Primary Responsibility**: Character set conversion interface
- **Key Definitions**:
  - Character set constants
  - Converter interface
  - Encoding detection types

### `scidvspc/src/dstring.cpp` (4,424 bytes)
- **Primary Responsibility**: Dynamic string handling
- **Key Functions**:
  - Dynamic string allocation
  - String concatenation
  - String encoding utilities

### `scidvspc/src/dstring.h` (2,458 bytes)
- **Primary Responsibility**: Dynamic string class definition
- **Key Definitions**:
  - DString class for dynamic string management
  - String manipulation methods
  - Used for text handling in encoding/decoding

### `scidvspc/src/stralloc.cpp` (3,800 bytes)
- **Primary Responsibility**: String allocator implementation
- **Key Functions**:
  - Fast bulk string allocation
  - Bucket-based memory management
  - Space-efficient string storage

### `scidvspc/src/stralloc.h` (4,408 bytes)
- **Primary Responsibility**: StrAllocator class definition
- **Key Definitions**:
  - StrAllocator class for bulk string allocation
  - Bucket-based allocation (default 32KB)
  - Minimized memory waste for many short strings
  - Used extensively in NameBase for player/event/site/round names

### `scidvspc/src/textbuf.cpp` (7,487 bytes)
- **Primary Responsibility**: Text buffer operations
- **Key Functions**:
  - Text buffer management
  - String encoding helpers
  - Text formatting

### `scidvspc/src/textbuf.h` (3,499 bytes)
- **Primary Responsibility**: Text buffer interface
- **Key Definitions**:
  - Text buffer structures
  - Buffer manipulation methods
  - String handling utilities

## Additional Support Files

### `scidvspc/src/matsig.cpp` (7,673 bytes)
- **Primary Responsibility**: Material signature calculation
- **Key Functions**:
  - Material position encoding
  - Signature generation
  - Material comparison

### `scidvspc/src/matsig.h` (8,404 bytes)
- **Primary Responsibility**: Material signature structures
- **Key Definitions**:
  - Material signature types
  - Signature calculation constants

### `scidvspc/src/strtree.h` (24,177 bytes)
- **Primary Responsibility**: String tree structures
- **Key Definitions**:
  - Compressed text storage
  - String compression algorithms
  - Tree-based text indexing

---

## Summary

**Total Files: 53**

### By Category:
- **Core Common Definitions**: 1 file (common.h)
- **Core Index Files (.si4)**: 4 files (index.cpp, index.h, date.h, date.cpp)
- **File I/O and Multi-byte**: 5 files (mfile.cpp, mfile.h, bytebuf.cpp, bytebuf.h, bytepack.h)
- **Name Processing (.sn4)**: 2 files (namebase.cpp, namebase.h)
- **Game File Processing (.sg4)**: 6 files (game.cpp, game.h, position.cpp, position.h, gfile.cpp, gfile.h)
- **Error Handling**: 3 files (error.h, myassert.cpp, myassert.h)
- **Utilities**: 14 files (misc.cpp/h, hash.h, attacks.h, sqlist.h, sqset.h, movelist.cpp/h, sqmove.h, optable.cpp/h, tokens.h, tree.cpp/h, crosstab.cpp/h)
- **Conversion Tools**: 5 files (pgnscid.cpp, scidt.cpp, scidlet.cpp, scmerge.cpp, eco2epd.cpp)
- **Specialized Parsing**: 7 files (pgnparse.cpp, pgnparse.h, recog.cpp, recog.h, nagtext.h, naglatex.h, stored.cpp/h)
- **Character/String**: 8 files (charsetconverter.cpp/h, charsetdetector.cpp/h, dstring.cpp/h, stralloc.cpp/h, textbuf.cpp/h)
- **Additional Support**: 3 files (matsig.cpp, matsig.h, strtree.h)

### Files NOT Included (Not related to database encoding/decoding):
- **GUI files**: tkscid.cpp/h, tk_selection.cpp, tkdnd/* (user interface, not DB format)
- **CQL files**: cql/* (Chess Query Language for searching, not DB format)
- **Polyglot files**: polyglot/* (opening book format, not SCID DB format)
- **EGTB files**: egtb/tbindex.cpp (endgame tablebase indexing, not DB format)
- **UniversalCharDet**: universalchardet/* (external library for charset detection)
- **zlib**: zlib/* (external compression library)
- **Engine**: engine.cpp/h (chess engine for analysis, not DB format)
- **PBook**: pbook.cpp/h (puzzle book format, not SCID DB format)
- **Probe**: probe.cpp/h (EGTB probing interface, not DB format)
- **Spellchk**: spellchk.cpp/h (name spell checking, not DB format)

### Critical Files for Understanding SCID Format:
1. **common.h** - Core constants, types, and version definitions
2. **index.cpp/h** - Understanding .si4 file structure and encoding
3. **namebase.cpp/h** - Understanding .sn4 front-coding compression
4. **game.cpp/h** - Understanding .sg4 move encoding (most complex)
5. **position.cpp/h** - Understanding piece tracking and move validation
6. **mfile.cpp** - Understanding big-endian multi-byte I/O
7. **bytebuf.cpp/h** - Understanding streaming byte buffer operations
8. **date.cpp/h** - Understanding packed date encoding
9. **tokens.h** - Understanding PGN tokenization
10. **movelist.cpp/h** - Understanding move representation

These 53 files work together to implement complete SCID database encoding and decoding system, handling all three file formats (.si4, .sn4, .sg4) with sophisticated compression and optimization algorithms.
