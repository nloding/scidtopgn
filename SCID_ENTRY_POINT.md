# SCID Decoding Entry Point

The entry point for decoding a SCID database is the process of opening and reading the three SCID files (.si4, .sn4, .sg4) in the correct sequence.

## Overview

A SCID database consists of three files with the same base name:
- **`.si4`** - Index file (game metadata, 182-byte header + 47-byte entries)
- **`.sn4`** - Name file (player/event/site/round names with front-coding)
- **`.sg4`** - Game file (move data, variations, comments in binary)

## Entry Point Flow

### 1. Main Entry Point: `scidt.cpp`

The canonical entry point is in `scidt.cpp`'s `main()` function:

```cpp
int main(int argc, char *argv[])
{
    progname = argv[0];
    filename = argv[2];  // e.g., "database" → reads database.si4, database.sn4, database.sg4
    option = argv[1];     // e.g., "-i" for info, "-l" for list
}
```

**Usage Example:**
```bash
scidt -i database    # Display general database information
scidt -l database    # List all games
```

### 2. Initialize Core Objects

From `scidt.cpp` main function:

```cpp
Index * idx = new Index;
NameBase * nb = new NameBase;

// Set base filename (adds .si4, .sn4, .sg4 automatically)
idx->SetFileName(filename);    // → opens database.si4
nb->SetFileName(filename);     // → opens database.sn4
// .sg4 is opened implicitly via GFile when needed
```

### 3. Open Index File (.si4)

```cpp
err = idx->OpenIndexFile(FMODE_ReadOnly);
```

**What this does:**
- Opens `database.si4`
- Reads and validates header (182 bytes)
  - Checks magic: `"Scid.si\0"`
  - Reads version: big-endian uint16 (should be 400)
  - Reads game count: big-endian uint24
  - Reads description, custom flags
- Prepares to read index entries

**From `index.cpp`:**
```cpp
struct indexHeaderT {
    char magic[9];      // "Scid.si"
    versionT version;    // 400 for SCID 4.0
    uint baseType;
    gameNumberT numGames;    // Total games
    gameNumberT autoLoad;
    char description[SCID_DESC_LENGTH + 1];
    char customFlagDesc[CUSTOM_FLAG_MAX][CUSTOM_FLAG_DESC_LENGTH+1];
};
```

### 4. Open Name File (.sn4)

```cpp
err = nb->ReadNameFile();
```

**What this does:**
- Opens `database.sn4`
- Reads name file header (36 bytes)
  - Magic: `"Scid.sn\0"`
  - Name counts: players, events, sites, rounds
  - Maximum frequencies
- Reads front-coded name records
- Builds name lookup tables

### 5. Accessing Game Data

When you want to decode a specific game:

#### 5a. Fetch Index Entry

```cpp
IndexEntry * entry = idx->FetchEntry(gameNumber);
```

**What this does:**
- If index not in memory: calls `idx->ReadEntries()` to load from .si4
- Returns pointer to `IndexEntry` containing:
  - `Offset` - byte position in .sg4 file
  - `Length` - game data length (17-bit value)
  - Player IDs (packed: white, black, event, site, round)
  - Date field (packed 32-bit: game date + event date)
  - Result, ELOs, ECO, etc.

**From `index.h`:**
```cpp
class IndexEntry {
private:
    uint Offset;              // Start of gamefile record
    uint Length_Low;          // Lower 16 bits of length
    byte Length_High;         // High bit + flags
    ushort WhiteID_Low;        // Lower 16 bits of White ID
    ushort BlackID_Low;        // Lower 16 bits of Black ID
    // ... (other packed fields)
public:
    inline uint GetOffset()  { return Offset; }
    inline uint GetLength()  { /* extract 17-bit value */ }
    inline idNumberT GetWhite();
    inline idNumberT GetBlack();
    // ... getter methods for all fields
};
```

#### 5b. Fetch Names

```cpp
const char * white = entry->GetWhiteName(nb);
const char * black = entry->GetBlackName(nb);
const char * event = entry->GetEventName(nb);
```

**What this does:**
- Looks up names from NameBase using IDs from index entry
- Names are stored with front-coding compression

#### 5c. Read Game Data

```cpp
GFile * gf = new GFile;
gf->Open(filename);  // Opens database.sg4

ByteBuffer * bb = new ByteBuffer;
err = gf->ReadGame(bb, entry->GetOffset(), entry->GetLength());
```

**What this does:**
- Calculates which 131,072-byte block contains the game
- Loads that block if not in cache
- Provides external pointer to game data in the block
- Sets ByteBuffer to point to game data

**From `gfile.cpp`:**
```cpp
errorT GFile::ReadGame(ByteBuffer * bb, uint offset, uint length)
{
    int blockNum = (offset / GF_BLOCKSIZE);
    int endBlockNum = (offset + length - 1) / GF_BLOCKSIZE;

    if (CurrentBlock->blockNum != blockNum) {
        Fetch(blockNum);  // Load correct block
    }

    bb->ProvideExternal(&(CurrentBlock->data[offset % GF_BLOCKSIZE]),
                         length);
    return OK;
}
```

### 6. Decode Game

```cpp
Game * game = new Game;
err = game->Decode(bb, GAME_DECODE_ALL);
```

**What this does:**
- Parses game structure from ByteBuffer
- **Step 1**: Skip PGN tags (until null terminator byte)
  ```cpp
  static errorT skipTags(ByteBuffer * buf) {
      byte b;
      b = buf->GetByte();
      while (b != 0) {  // Null terminator = end of tags
          if (b == 255) { buf->Skip(3); }  // EventDate
          else if (b > MAX_TAG_LEN) { /* common tag */ }
          else { /* regular tag [len][name][len][value] */ }
          b = buf->GetByte();
      }
      return OK;
  }
  ```
- **Step 2**: Read game flags (1 byte)
- **Step 3**: Read FEN if non-standard start flag set
- **Step 4**: Decode move sequence
  - Reads bytes and interprets based on piece number
  - Handles piece-specific decoding:
    - `decodeKing()` - values 0-10 (sqdiff table)
    - `decodeQueen()` - rook moves (1 byte) or diagonal (2 bytes)
    - `decodePawn()` - promotions, double push
    - etc.
  - Handles variations, NAGs, comments
- **Step 5**: Validates end-of-game marker (0x0F)

**From `game.cpp`:**
```cpp
errorT Game::Decode(ByteBuffer * buf, uint flags)
{
    // Skip tags
    err = skipTags(buf);

    // Read flags
    byte gameFlags = buf->GetByte();
    NonStandardStart = (gameFlags & 1) != 0;
    PromotionsFlag = (gameFlags & 2) != 0;

    // Read optional FEN for non-standard start
    if (NonStandardStart) {
        char * tempStr;
        buf->GetTerminatedString(&tempStr);
        StartPos->ReadFromFEN(tempStr);
    }

    // Decode moves
    return DecodeMainLine(buf);
}

static errorT decodeKing(ByteBuffer * buf, simpleMoveT * sm)
{
    byte val = buf->GetByte();
    ASSERT(sm->pieceNum == 0);  // King is always piece 0

    static const int sqdiff[] = {
        0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2
    };
    if (val == 0) {
        sm->to = sm->from;  // Null move
    } else if (val < 1 || val > 10) {
        return ERROR_Decode;
    }
    sm->to = sm->from + sqdiff[val];
    return OK;
}
```

### 7. Update Position

```cpp
Position * pos = new Position;
pos->StdStart();  // Initialize standard piece numbers

for each move in game:
    pos->DoSimpleMove(decodedMove);
    // - Tracks piece numbers via List[color][16]
    // - Handles capture swap algorithm
    // - Validates move legality
```

**From `position.cpp`:**
```cpp
errorT Position::DoSimpleMove(simpleMoveT * sm)
{
    // Find moving piece
    squareT fromSq = List[ToMove][sm->pieceNum];

    // Handle capture
    if (sm->capturedPiece != EMPTY) {
        sm->capturedNum = ListPos[sm->capturedSquare];
        Count[enemy]--;

        // SWAP: Move last piece into captured slot
        ListPos[List[enemy][Count[enemy]]] = sm->capturedNum;
        List[enemy][sm->capturedNum] = List[enemy][Count[enemy]];
    }

    // Update piece position
    List[ToMove][sm->pieceNum] = sm->to;
    ListPos[sm->to] = sm->pieceNum;

    // Handle promotion, castling, etc.
    // ...
}
```

## Complete Entry Point Sequence

```
┌────────────────────────────────────────────────────────────┐
│ 1. Command Line: scidt -i database             │
└────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────────┐
│ 2. Initialization                               │
│    idx->SetFileName("database")  → .si4         │
│    nb->SetFileName("database")   → .sn4         │
└────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────────┐
│ 3. Open Index File                             │
│    idx->OpenIndexFile(FMODE_ReadOnly)            │
│    Reads 182-byte header + index entries          │
│    Validates magic, version                          │
└────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────────┐
│ 4. Open Name File                              │
│    nb->ReadNameFile()                            │
│    Reads 36-byte header + front-coded names        │
└────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────────┐
│ 5. For Each Game:                               │
│                                                    │
│  ┌─────────────────────────────────────────────┐   │
│  │ 5a. Fetch Index Entry                 │   │
│  │ entry = idx->FetchEntry(gameNum)       │   │
│  │                                     │   │
│  │ - Get offset: entry->GetOffset()      │   │
│  │ - Get length: entry->GetLength()      │   │
│  │ - Get IDs: entry->GetWhite(), etc.   │   │
│  └─────────────────┬───────────────────────┘   │
│                    │                             │
│  ┌────────────────┴────────────────┐         │
│  │ 5b. Fetch Names                │         │
│  │ white = nb->GetName(WHITE_ID)   │         │
│  │ black = nb->GetName(BLACK_ID)   │         │
│  └──────────────────┬─────────────────┘         │
│                     │                             │
│  ┌────────────────┴────────────────┐         │
│  │ 5c. Read Game Data            │         │
│  │ gf->ReadGame(bb, offset, len) │         │
│  │                                  │         │
│  │ Returns pointer to game bytes  │         │
│  └──────────────────┬─────────────────┘         │
│                     │                             │
│  ▼                             ▼         │
│  ┌─────────────────────────────────────────┐         │
│  │ 6. Decode Game                  │         │
│  │ game->Decode(bb)                │         │
│  │                                  │         │
│  │ - Skip tags (until 0x00)      │         │
│  │ - Read flags (1 byte)            │         │
│  │ - Read FEN if needed            │         │
│  │ - Decode moves (piece-specific)   │         │
│  │ - Handle variations, NAGs,        │         │
│  │   comments                         │         │
│  └──────────────────┬─────────────────┘         │
│                     │                             │
│  ▼                             ▼         │
│  ┌─────────────────────────────────────────┐         │
│  │ 7. Update Position               │         │
│  │ pos->DoSimpleMove(move)           │         │
│  │                                  │         │
│  │ - Track piece numbers            │         │
│  │ - Handle capture swap            │         │
│  │ - Validate moves                │         │
│  └───────────────────────────────────┘         │
└───────────────────────────────────────────────────────┘
```

## Minimal Decoding Entry Point

For a minimal SCID decoder, you need:

```cpp
// 1. Include headers
#include "index.h"
#include "namebase.h"
#include "gfile.h"
#include "game.h"
#include "position.h"
#include "bytebuf.h"

// 2. Open files
Index * idx = new Index;
NameBase * nb = new NameBase;
GFile * gf = new GFile;

idx->SetFileName("database");
nb->SetFileName("database");
gf->Open("database");

// 3. Open index and name files
idx->OpenIndexFile(FMODE_ReadOnly);
nb->ReadNameFile();

// 4. Decode each game
for (gameNumberT gnum = 0; gnum < idx->GetNumGames(); gnum++) {

    // Get index entry
    IndexEntry * entry = idx->FetchEntry(gnum);

    // Get names
    const char * white = entry->GetWhiteName(nb);
    const char * black = entry->GetBlackName(nb);

    // Read game data
    ByteBuffer bb;
    gf->ReadGame(&bb, entry->GetOffset(), entry->GetLength());

    // Decode game
    Game game;
    game.Decode(&bb, GAME_DECODE_ALL);

    // Access moves
    // Game is now fully decoded with move tree, variations, annotations
}
```

## Key Entry Point Files

| File | Purpose | Entry Point Function |
|------|----------|---------------------|
| `scidt.cpp` | Command-line interface | `main()` |
| `index.cpp` | Index file handling | `Index::OpenIndexFile()` |
| `namebase.cpp` | Name file handling | `NameBase::ReadNameFile()` |
| `gfile.cpp` | Game file handling | `GFile::ReadGame()` |
| `game.cpp` | Game decoding | `Game::Decode()` |
| `position.cpp` | Position tracking | `Position::DoSimpleMove()` |
| `bytebuf.h` | Byte stream access | `ByteBuffer::GetByte()` |

## File Opening Order

**Critical**: Files must be opened in this order:

1. **First**: Open `.si4` (index) - Read header, get game count
2. **Second**: Open `.sn4` (names) - Read header, build name tables
3. **Third**: `.sg4` (games) is opened on-demand when reading specific games

**Why this order?**
- `.si4` header tells you how many games exist
- `.si4` contains offsets into `.sg4` file
- `.sn4` provides names referenced by IDs in `.si4`
- `.sg4` is read block-by-block as needed for memory efficiency

## Summary

The entry point for SCID decoding is a **multi-stage process**:

1. **Command-line parsing** → Base filename + options
2. **Object initialization** → Index, NameBase, GFile objects
3. **Index file opening** → Read header, prepare to read entries
4. **Name file opening** → Read front-coded names
5. **Per-game loop**:
   - Fetch index entry (metadata + offset + length)
   - Lookup names from NameBase
   - Read game data block from GFile
   - Decode game structure
   - Track position state

All decoding logic is distributed across 6 core files:
- `index.cpp` - Metadata (.si4)
- `namebase.cpp` - Names (.sn4)
- `gfile.cpp` - Game data blocks (.sg4)
- `game.cpp` - Move decoding
- `position.cpp` - Position tracking
- `bytebuf.cpp/h` - Byte stream operations
