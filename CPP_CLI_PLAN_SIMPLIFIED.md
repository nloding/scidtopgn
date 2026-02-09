# Plan: Standalone C++ SCID-to-PGN CLI Tool (SIMPLIFIED)

**⚠️ This is the SIMPLIFIED plan - leverages existing SCID PGN output functionality**

- **Original plan** (custom PGN implementation): `CPP_CLI_PLAN.md` (750 lines new code)
- **This plan** (use existing `Game::WriteToPGN()`): Only 240 lines new code ✅

## Overview

Create a standalone C++ CLI tool (`scidtopgn`) that reads SCID database files (.si4, .sn4, .sg4) and outputs PGN format to stdout.

**Key Constraint**: NO modifications to original C++ files. Only create NEW files.

**Critical Discovery**: The existing `Game` class has full PGN output functionality via `WriteToPGN(TextBuffer*)`!

---

## Phase 1: Architecture

### Data Flow

```
SCID Files (.si4, .sn4, .sg4)
         ↓
  Index & NameBase (metadata)
         ↓
      Game file (.sg4)
         ↓
    Game::Decode() (decodes moves)
         ↓
    Game::LoadStandardTags() (loads metadata)
         ↓
    Game::WriteToPGN() (outputs to TextBuffer)
         ↓
     TextBuffer::GetBuffer()
         ↓
         stdout
```

### Key Classes/Methods from Existing Code

| Class | Method | Purpose |
|-------|--------|---------|
| `Index` | `OpenIndexFile(FMODE_ReadOnly)` | Open .si4 file |
| `NameBase` | `ReadNameFile()` | Open .sn4 file, load names |
| `GFile` | `Open()` | Open .sg4 file |
| `GFile` | `ReadGame(ByteBuffer*, offset, length)` | Read game data |
| `Game` | `Decode(ByteBuffer*, flags)` | Decode moves/game |
| `Game` | `LoadStandardTags(IndexEntry*, NameBase*)` | Load metadata from index |
| `Game` | `WriteToPGN(TextBuffer*)` | **Convert to PGN and write to buffer** |
| `TextBuffer` | `GetBuffer()` | Get buffer content |
| `TextBuffer` | `NewLine()` | Add newline |

---

## Phase 2: Project Structure

```
scidtopgn/
├── src/
│   └── main.cpp                 # NEW - CLI entry point (~150 lines)
├── lib/                         # Compile original SCID code here
├── CMakeLists.txt                # NEW - Build configuration (~60 lines)
└── README.md                     # NEW - Usage documentation (~30 lines)
```

**Total New Code: ~240 lines** (vs ~750 lines in original plan!)

---

## Phase 3: New Files to Create

### 1. `src/main.cpp` (Entry Point)

```cpp
#include "index.h"
#include "namebase.h"
#include "gfile.h"
#include "game.h"
#include "textbuf.h"
#include "error.h"

#include <iostream>
#include <cstring>

int main(int argc, char* argv[]) {
    if (argc != 2) {
        std::cerr << "Usage: scidtopgn <database>" << std::endl;
        return 1;
    }

    const char* database_path = argv[1];

    // Initialize SCID objects
    Index* idx = new Index();
    NameBase* nb = new NameBase();
    GFile* gf = new GFile();

    errorT err;

    // Set filename (adds .si4, .sn4, .sg4 automatically)
    idx->SetFileName(database_path);
    nb->SetFileName(database_path);

    // Open index file (.si4)
    err = idx->OpenIndexFile(FMODE_ReadOnly);
    if (err != OK) {
        std::cerr << "Error: cannot open index file (.si4)" << std::endl;
        return 1;
    }

    // Open name file (.sn4)
    err = nb->ReadNameFile();
    if (err != OK) {
        std::cerr << "Error: cannot read name file (.sn4)" << std::endl;
        return 1;
    }

    // Open game file (.sg4)
    err = gf->Open(database_path);
    if (err != OK) {
        std::cerr << "Error: cannot open game file (.sg4)" << std::endl;
        return 1;
    }

    // Process each game
    gameNumberT numGames = idx->GetNumGames();

    for (gameNumberT gnum = 0; gnum < numGames; gnum++) {
        // Fetch index entry
        IndexEntry* entry = idx->FetchEntry(gnum);
        if (!entry) {
            continue;
        }

        // Skip deleted games
        if (entry->GetDeleteFlag()) {
            continue;
        }

        // Read game data from .sg4
        ByteBuffer bb;
        err = gf->ReadGame(&bb, entry->GetOffset(), entry->GetLength());
        if (err != OK) {
            std::cerr << "Warning: error reading game " << (gnum + 1) << std::endl;
            continue;
        }

        // Decode game (moves, variations, comments, NAGs)
        Game game;
        err = game.Decode(&bb, GAME_DECODE_ALL);
        if (err != OK) {
            std::cerr << "Warning: error decoding game " << (gnum + 1) << std::endl;
            continue;
        }

        // Load standard tags from IndexEntry and NameBase
        // This sets: Event, Site, Date, Round, White, Black, Result,
        //           WhiteElo, BlackElo, ECO code, etc.
        err = game.LoadStandardTags(entry, nb);
        if (err != OK) {
            std::cerr << "Warning: error loading tags for game " << (gnum + 1) << std::endl;
            continue;
        }

        // Set PGN format and style
        game.SetPgnFormat(PGN_FORMAT_Plain);  // Plain PGN
        game.SetPgnStyle(PGN_STYLE_TAGS        // Include all tags
                      | PGN_STYLE_COMMENTS    // Include comments
                      | PGN_STYLE_VARS        // Include variations
                      | PGN_STYLE_SYMBOLS);   // Use symbols (!?, etc.)

        // Write game to PGN format into TextBuffer
        TextBuffer tb;
        err = game.WriteToPGN(&tb);
        if (err != OK) {
            std::cerr << "Warning: error converting game " << (gnum + 1) << " to PGN" << std::endl;
            continue;
        }

        // Output to stdout
        std::cout << tb.GetBuffer();
    }

    // Cleanup
    delete idx;
    delete nb;
    gf->Close();
    delete gf;

    return 0;
}
```

### 2. `CMakeLists.txt` (Build Configuration)

```cmake
cmake_minimum_required(VERSION 3.15)
project(scidtopgn VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Source directories
set(SCID_SRC_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../scidvspc/scidvspc-main/src")
set(SRC_DIR "${CMAKE_CURRENT_SOURCE_DIR}/src")

# SCID source files (from original codebase - READ ONLY)
set(SCID_SOURCES
    ${SCID_SRC_DIR}/common.h
    ${SCID_SRC_DIR}/error.h
    ${SCID_SRC_DIR}/mfile.cpp
    ${SCID_SRC_DIR}/mfile.h
    ${SCID_SRC_DIR}/bytebuf.cpp
    ${SCID_SRC_DIR}/bytebuf.h
    ${SCID_SRC_DIR}/index.cpp
    ${SCID_SRC_DIR}/index.h
    ${SCID_SRC_DIR}/namebase.cpp
    ${SCID_SRC_DIR}/namebase.h
    ${SCID_SRC_DIR}/game.cpp
    ${SCID_SRC_DIR}/game.h
    ${SCID_SRC_DIR}/gfile.cpp
    ${SCID_SRC_DIR}/gfile.h
    ${SCID_SRC_DIR}/position.cpp
    ${SCID_SRC_DIR}/position.h
    ${SCID_SRC_DIR}/date.cpp
    ${SCID_SRC_DIR}/date.h
    ${SCID_SRC_DIR}/misc.cpp
    ${SCID_SRC_DIR}/misc.h
    ${SCID_SRC_DIR}/stralloc.cpp
    ${SCID_SRC_DIR}/stralloc.h
    ${SCID_SRC_DIR}/sqmove.h
    ${SCID_SRC_DIR}/movelist.cpp
    ${SCID_SRC_DIR}/movelist.h
    ${SCID_SRC_DIR}/sqlist.h
    ${SCID_SRC_DIR}/sqset.h
    ${SCID_SRC_DIR}/attacks.h
    ${SCID_SRC_DIR}/textbuf.cpp
    ${SCID_SRC_DIR}/textbuf.h
)

# New source files
set(TOOL_SOURCES
    ${SRC_DIR}/main.cpp
)

# Include directories
include_directories(
    ${SCID_SRC_DIR}
    ${SRC_DIR}
)

# Create static library from SCID code
add_library(scidlib STATIC ${SCID_SOURCES})

# Create executable
add_executable(scidtopgn ${TOOL_SOURCES})

# Link SCID library
target_link_libraries(scidtopgn scidlib)

# Installation
install(TARGETS scidtopgn DESTINATION bin)
```

### 3. `README.md` (Usage Documentation)

```markdown
# scidtopgn - SCID to PGN Converter

A standalone C++ CLI tool that converts SCID chess database files to PGN format.

## Usage

```bash
scidtopgn <database_path>
```

Where `<database_path>` is the base name of SCID database (without file extensions).

### Example

```bash
scidtopgn mygames
```

This reads `mygames.si4`, `mygames.sn4`, and `mygames.sg4` and outputs PGN to stdout.

### Save to File

```bash
scidtopgn mygames > output.pgn
```

### Example Output

```pgn
[Event "World Championship"]
[Site "London"]
[Date "2023.04.15"]
[Round "1"]
[White "Magnus Carlsen"]
[Black "Fabiano Caruana"]
[Result "1-0"]
[WhiteElo "2853"]
[BlackElo "2820"]
[ECO "C50"]

1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 d6 6. c3 a6 7. a4 Ba7
8. Re1 h6 9. h3 O-O 10. Nbd2 Be6 11. Bxe6 fxe6 12. b4 Nd7 13. Qb3 Nb6 14. a5 Nc4
15. Nxc4 Bxc4 16. d4 exd4 17. cxd4 Bxd4 18. Nxd4 Qxd4 19. Rd1 Qc5 20. Qxc5 Nxc5
21. Rxe6 Rad8 22. Re7 Kh8 23. Rxa7 Rxd4 24. Rxa6 Nxe4 25. Ra8+ Kg7 26. Rxe8 Rd1+
27. Kh2 Rd7 28. b5 c6 29. Rc8 Rxb5 30. Rxc6 Rb2 31. Kg3 Nxg5 32. fxg5 Rxg5+
33. Kf4 g5+ 34. Ke3 Rf5 35. Rd8 h5 36. Rd3 Rf1 37. Kf3 Kf6 38. Kg4 h4
39. Rd6+ Ke5 40. Rd8 Rf7 41. Rd5+ Ke6 42. Rd8 Ke7 43. Kh5 Rf5 44. Kh4 Rxg5+
45. Kxg5 Kf7 46. Rd6 h3 47. g3 g6 48. Rb6 Kg7 49. Kf4 Kf7 50. Ke5 Kg7
51. Kd6 Kf8 52. Rd7 Ke8 53. Kc7 1-0
```

## Features

- ✅ Reads SCID database files (.si4, .sn4, .sg4)
- ✅ Converts all games to standard PGN format
- ✅ Includes all PGN tags (Event, Site, Date, Round, White, Black, Result, ELO, ECO, etc.)
- ✅ Handles variations (with parentheses)
- ✅ Includes comments (in braces)
- ✅ Supports NAGs (Numeric Annotation Glyphs) like !, ?, !?
- ✅ Handles special positions (FEN tags for non-standard starts)
- ✅ Skips deleted games
- ✅ Uses original SCID PGN output logic (100% compatible)

## Building

```bash
mkdir build
cd build
cmake ..
make
```

## Requirements

- C++17 or later
- CMake 3.15 or later
- SCID source code in `../scidvspc/scidvspc-main/src/`

## PGN Features

The tool outputs full PGN format including:

### Standard Tags
- Event, Site, Date, Round, White, Black, Result
- WhiteElo, BlackElo
- ECO code
- Additional custom tags if present

### Move Notation
- Standard Algebraic Notation (SAN)
- Move numbers (1. e4 e5 2. Nf3...)
- Check and mate symbols (+ and #)

### Annotations
- Comments in braces: {excellent move}
- NAGs: !, ?, !!, ??, !?, $1-$255
- Variations in parentheses: (1. e4 e5 2. Nf3 (2. d4) Nc6)

### Special Cases
- Castling: O-O, O-O-O
- Pawn promotions: e8=Q
- En passant: exd6
- Non-standard starting positions: [FEN "..."]

## License

Uses SCID source code. Original SCID license applies to reused code.
```

---

## Phase 4: Implementation Details

### Key Discovery: Existing PGN Output

The `Game` class already has complete PGN output functionality:

#### Method: `Game::WriteToPGN(TextBuffer* tb)`

**Location**: `game.cpp` line 3998

**What it does**:
1. Outputs PGN tags (Event, Site, Date, Round, White, Black, Result, etc.)
2. Outputs moves in SAN notation
3. Handles variations and comments
4. Includes NAGs
5. Adds result at end

#### Method: `Game::LoadStandardTags(IndexEntry* ie, NameBase* nb)`

**Location**: `game.h` line 455

**What it does**:
- Loads all standard tag strings from IndexEntry and NameBase
- Sets: EventStr, SiteStr, Date, RoundStr, WhiteStr, BlackStr
- Sets: Result, WhiteElo, BlackElo, ECO code, etc.

### PGN Style Options

From `game.h` lines 192-210:

```cpp
enum {
    PGN_FORMAT_Plain = 0,
    PGN_FORMAT_HTML = 1,
    PGN_FORMAT_Latex = 2,
    PGN_FORMAT_Color = 3
};

#define PGN_STYLE_TAGS             1   // Include all tags
#define PGN_STYLE_COMMENTS         2   // Include comments
#define PGN_STYLE_VARS             4   // Include variations
#define PGN_STYLE_SYMBOLS         32   // Use symbols (!, ?, etc.)
#define PGN_STYLE_SHORT_HEADER    64   // Compact header format
```

### TextBuffer Usage

`TextBuffer` class (defined in `textbuf.h`):
- `GetBuffer()` - Returns `char*` to buffer content
- `PrintString(const char* str)` - Add text to buffer
- `NewLine()` - Add newline
- `DumpToFile(FILE* fp)` - Write buffer to file (not used, we use GetBuffer())

---

## Phase 5: Build and Test

### Build Steps

```bash
# 1. Create build directory
cd /home/nloding/code/scidtopgn
mkdir build
cd build

# 2. Configure with CMake
cmake ..

# 3. Build
make

# 4. Test with sample database
./scidtopgn ../test_database > test_output.pgn

# 5. Verify output
head -20 test_output.pgn
```

### Test Checklist

- [ ] Opens SCID database files correctly
- [ ] Decodes all games without errors
- [ ] Outputs valid PGN format
- [ ] Includes all standard tags
- [ ] Handles variations (parentheses)
- [ ] Includes comments (braces)
- [ ] Outputs NAGs (!?, etc.)
- [ ] Handles non-standard starting positions (FEN tag)
- [ ] Skips deleted games
- [ ] Output is readable by chess engines/tools

---

## Phase 6: Optional Enhancements

### Command-Line Options

```bash
scidtopgn [options] <database>

Options:
  -n <number>     Export specific game number
  -r <range>     Export game range (e.g., 1-100, 200-)
  -o <file>      Output to file instead of stdout
  --no-comments   Exclude comments
  --no-vars      Exclude variations
  --short-header  Use compact header format
  -v             Verbose output (show progress)
```

### Performance Optimizations

For large databases:

1. **Batch reading**: Read multiple games at once
2. **Memory mapping**: Use mmap for large files
3. **Parallel processing**: Use threads for multi-core systems

```cpp
// Example: Progress reporting
#include <sys/ioctl.h>
#include <unistd.h>

void reportProgress(gameNumberT current, gameNumberT total) {
    int progress = (current * 100) / total;
    fprintf(stderr, "\rProgress: %d%% (%d/%d games)",
            progress, current, total);
    fflush(stderr);
}
```

### Error Recovery

```cpp
// Enhanced error handling
struct Stats {
    int totalGames;
    int successful;
    int skippedDeleted;
    int errors;
};

Stats stats = {0};
stats.totalGames = numGames;

for (gameNumberT gnum = 0; gnum < numGames; gnum++) {
    if (entry->GetDeleteFlag()) {
        stats.skippedDeleted++;
        continue;
    }

    err = game.Decode(&bb, GAME_DECODE_ALL);
    if (err != OK) {
        std::cerr << "Warning: game " << (gnum + 1)
                  << " decode error: " << err << std::endl;
        stats.errors++;
        continue;
    }

    err = game.WriteToPGN(&tb);
    if (err != OK) {
        std::cerr << "Warning: game " << (gnum + 1)
                  << " PGN error: " << err << std::endl;
        stats.errors++;
        continue;
    }

    std::cout << tb.GetBuffer();
    stats.successful++;
}

// Report summary
std::cerr << "\n\nSummary:\n";
std::cerr << "  Total games: " << stats.totalGames << "\n";
std::cerr << "  Successful: " << stats.successful << "\n";
std::cerr << "  Skipped (deleted): " << stats.skippedDeleted << "\n";
std::cerr << "  Errors: " << stats.errors << "\n";
```

---

## Phase 7: Technical Notes

### Memory Management

- **SCID uses new/delete**: Always match pattern
- **TextBuffer**: Manages its own buffer (don't free GetBuffer())
- **Game object**: Can be reused or recreated per game
- **ByteBuffer**: Managed by SCID code

### String Handling

- **NameBase names**: Don't free (managed by NameBase)
- **Game strings**: Game manages its own strings
- **TextBuffer content**: Managed by TextBuffer

### File Handle Management

```cpp
// Always close files in correct order
delete idx;      // Closes .si4
delete nb;       // Closes .sn4
gf->Close();     // Closes .sg4
delete gf;
```

### PGN Format Validation

The output follows PGN standard (Portable Game Notation):
- Tags: `[Name "Value"]`
- Moves: Standard Algebraic Notation
- Comments: `{text}`
- Variations: `(moves)`
- NAGs: `$1` to `$255`, or symbols `!`, `?`, `!!`, etc.

---

## Summary

### Files to Create

| File | Lines | Purpose |
|------|-------|---------|
| `src/main.cpp` | ~150 | CLI entry point, game loop |
| `CMakeLists.txt` | ~60 | Build configuration |
| `README.md` | ~30 | Documentation |

**Total New Code: ~240 lines**

### Files Reused (No Modification): ~20 files

| Category | Files |
|----------|-------|
| Core | `index.cpp/h`, `namebase.cpp/h`, `game.cpp/h` |
| Files | `gfile.cpp/h`, `bytebuf.cpp/h`, `mfile.cpp/h` |
| Position | `position.cpp/h`, `movelist.cpp/h` |
| Support | `textbuf.cpp/h`, `date.cpp/h`, `misc.cpp/h`, `stralloc.cpp/h` |
| Utils | `attacks.h`, `sqmove.h`, `sqlist.h`, `sqset.h` |

### Why This Plan is Better

1. **Less code**: 240 lines vs 750 lines
2. **No PGN implementation needed**: Use existing `Game::WriteToPGN()`
3. **No SAN notation conversion needed**: Already implemented
4. **No move tree traversal needed**: Already implemented
5. **Guaranteed compatibility**: Uses SCID's own PGN output
6. **Easier to maintain**: Fewer files, simpler code
7. **Faster to implement**: Leverage existing tested code

### Success Criteria

- ✅ Opens SCID database files correctly
- ✅ Decodes all games without errors
- ✅ Outputs standard-compliant PGN
- ✅ Handles all PGN features (variations, comments, NAGs)
- ✅ Preserves all metadata (tags)
- ✅ No modifications to original SCID code
- ✅ Simple to build and use

---

## Comparison: Original Plan vs Simplified Plan

| Aspect | Original Plan | Simplified Plan |
|--------|--------------|----------------|
| New files | 5 files | 3 files |
| New code | ~750 lines | ~240 lines |
| PGN implementation | Custom (complex) | Existing (tested) |
| SAN notation | Custom (error-prone) | Existing (verified) |
| Move tree traversal | Custom | Existing |
| Maintenance burden | High | Low |
| Implementation time | Days | Hours |
| Code quality | Uncertain | Proven |
