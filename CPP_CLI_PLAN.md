# Plan: Standalone C++ SCID-to-PGN CLI Tool (OBSOLETE)

## ⚠️ THIS PLAN IS OBSOLETE

**Please use the simplified plan instead: `CPP_CLI_PLAN_SIMPLIFIED.md`**

The simplified plan leverages existing SCID PGN output functionality (`Game::WriteToPGN()`), reducing new code from 750 lines to 240 lines and eliminating the need to implement custom PGN conversion logic.

---

## Overview (Original Plan - OBSOLETE)

---

## Phase 1: Project Structure

### New File Layout

```
scidtopgn/
├── src/
│   ├── main.cpp                 # NEW - CLI entry point
│   ├── pgn_converter.cpp         # NEW - Game to PGN conversion
│   └── pgn_converter.h           # NEW - PGN converter interface
├── include/                      # NEW - Headers for original SCID code
│   └── (symlinks or copies of .h files)
├── lib/                          # NEW - Object files from original code
├── CMakeLists.txt                # NEW - Build configuration
└── README.md                     # NEW - Usage documentation
```

---

## Phase 2: Original SCID Files to Reuse

### Core Files (No Modification)

These files will be compiled as static library components:

| File | Purpose | Reuse For |
|------|---------|-----------|
| `common.h` | Core definitions | Constants, types, error codes |
| `error.h` | Error handling | Error codes and macros |
| `mfile.cpp/h` | Multi-byte I/O | Big-endian reads/writes |
| `bytebuf.cpp/h` | Byte stream | Buffer operations |
| `index.cpp/h` | Index file (.si4) | Game metadata, offsets |
| `namebase.cpp/h` | Name file (.sn4) | Player/event/site/round names |
| `game.cpp/h` | Game decoding | Move decoding, variations |
| `gfile.cpp/h` | Game file (.sg4) | Block-based game reading |
| `position.cpp/h` | Position tracking | Piece list, move execution |
| `date.cpp/h` | Date handling | Packed date encoding/decoding |
| `misc.cpp/h` | Utilities | String helpers, formatting |
| `stralloc.cpp/h` | String allocation | Efficient string storage |
| `sqmove.h` | Square/move helpers | Square calculations |
| `movelist.cpp/h` | Move list | Move storage and iteration |
| `sqlist.h` | Square list | Square collections |
| `sqset.h` | Square set | O(1) square membership |

### Support Files (Optional for Minimal Implementation)

| File | Purpose | When Needed |
|------|---------|-------------|
| `attacks.h` | Attack tables | For move validation |
| `hash.h` | Zobrist hashes | If using position hashing |
| `textbuf.cpp/h` | Text buffer | For formatted output |
| `dstring.cpp/h` | Dynamic strings | For string building |

---

## Phase 3: New Files to Create

### 1. `src/main.cpp` (Entry Point)

```cpp
#include "index.h"
#include "namebase.h"
#include "gfile.h"
#include "game.h"
#include "pgn_converter.h"

#include <iostream>
#include <cstdlib>

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

    // Set filename
    idx->SetFileName(database_path);
    nb->SetFileName(database_path);

    // Open index file (.si4)
    err = idx->OpenIndexFile(FMODE_ReadOnly);
    if (err != OK) {
        std::cerr << "Error opening index file: " << err << std::endl;
        return 1;
    }

    // Open name file (.sn4)
    err = nb->ReadNameFile();
    if (err != OK) {
        std::cerr << "Error reading name file: " << err << std::endl;
        return 1;
    }

    // Open game file (.sg4)
    err = gf->Open(database_path);
    if (err != OK) {
        std::cerr << "Error opening game file: " << err << std::endl;
        return 1;
    }

    // Process each game
    gameNumberT numGames = idx->GetNumGames();

    for (gameNumberT gnum = 0; gnum < numGames; gnum++) {
        // Fetch index entry
        IndexEntry* entry = idx->FetchEntry(gnum);
        if (!entry) {
            std::cerr << "Error fetching game " << gnum << std::endl;
            continue;
        }

        // Skip deleted games
        if (entry->GetDeleteFlag()) {
            continue;
        }

        // Read game data
        ByteBuffer bb;
        err = gf->ReadGame(&bb, entry->GetOffset(), entry->GetLength());
        if (err != OK) {
            std::cerr << "Error reading game " << gnum << std::endl;
            continue;
        }

        // Decode game
        Game game;
        err = game.Decode(&bb, GAME_DECODE_ALL);
        if (err != OK) {
            std::cerr << "Error decoding game " << gnum << std::endl;
            continue;
        }

        // Convert to PGN and output
        PGNConverter converter;
        converter.SetNameBase(nb);
        std::string pgn = converter.ConvertGame(&game, entry, gnum + 1);
        std::cout << pgn << std::endl;
    }

    // Cleanup
    delete idx;
    delete nb;
    gf->Close();
    delete gf;

    return 0;
}
```

### 2. `src/pgn_converter.h` (PGN Converter Interface)

```cpp
#ifndef PGN_CONVERTER_H
#define PGN_CONVERTER_H

#include "index.h"
#include "namebase.h"
#include "game.h"
#include "date.h"

#include <string>

class PGNConverter {
public:
    PGNConverter();
    ~PGNConverter();

    void SetNameBase(NameBase* nb);
    std::string ConvertGame(Game* game, IndexEntry* entry, gameNumberT gameNum);

private:
    NameBase* namebase_;

    // PGN tag generation
    std::string GenerateEventTag(IndexEntry* entry, gameNumberT gameNum);
    std::string GenerateSiteTag(IndexEntry* entry);
    std::string GenerateDateTag(IndexEntry* entry);
    std::string GenerateRoundTag(IndexEntry* entry);
    std::string GenerateWhiteTag(IndexEntry* entry);
    std::string GenerateBlackTag(IndexEntry* entry);
    std::string GenerateResultTag(IndexEntry* entry);

    // PGN move notation
    std::string ConvertMoveToSAN(simpleMoveT* move, Position* pos);
    std::string ConvertMoveTree(Game* game);

    // Helpers
    std::string SquareToString(squareT sq);
    std::string PieceToString(pieceT piece);
    std::string ResultToString(resultT result);
    std::string FormatDate(dateT date);
};

#endif // PGN_CONVERTER_H
```

### 3. `src/pgn_converter.cpp` (PGN Converter Implementation)

```cpp
#include "pgn_converter.h"
#include "sqmove.h"
#include "position.h"
#include <sstream>
#include <iomanip>

PGNConverter::PGNConverter() : namebase_(nullptr) {}

PGNConverter::~PGNConverter() {}

void PGNConverter::SetNameBase(NameBase* nb) {
    namebase_ = nb;
}

std::string PGNConverter::ConvertGame(Game* game, IndexEntry* entry, gameNumberT gameNum) {
    std::ostringstream pgn;

    // Generate PGN tags
    pgn << "[Event \"" << GenerateEventTag(entry, gameNum) << "\"]\n";
    pgn << "[Site \"" << GenerateSiteTag(entry) << "\"]\n";
    pgn << "[Date \"" << GenerateDateTag(entry) << "\"]\n";
    pgn << "[Round \"" << GenerateRoundTag(entry) << "\"]\n";
    pgn << "[White \"" << GenerateWhiteTag(entry) << "\"]\n";
    pgn << "[Black \"" << GenerateBlackTag(entry) << "\"]\n";
    pgn << "[Result \"" << GenerateResultTag(entry) << "\"]\n";

    // Add ELO tags if available
    eloT whiteElo = entry->GetWhiteElo();
    eloT blackElo = entry->GetBlackElo();
    if (whiteElo > 0) {
        pgn << "[WhiteElo \"" << whiteElo << "\"]\n";
    }
    if (blackElo > 0) {
        pgn << "[BlackElo \"" << blackElo << "\"]\n";
    }

    // Add ECO tag if available
    ecoT eco = entry->GetEcoCode();
    if (eco != ECO_None) {
        pgn << "[ECO \"" << eco << "\"]\n";
    }

    pgn << "\n";

    // Convert move tree to PGN
    pgn << ConvertMoveTree(game);

    // Add result
    pgn << " " << GenerateResultTag(entry) << "\n\n";

    return pgn.str();
}

std::string PGNConverter::GenerateEventTag(IndexEntry* entry, gameNumberT gameNum) {
    idNumberT eventID = entry->GetEvent();
    if (eventID != 0 && namebase_) {
        return std::string(namebase_->GetName(NAME_EVENT, eventID));
    }
    return "?";
}

std::string PGNConverter::GenerateSiteTag(IndexEntry* entry) {
    idNumberT siteID = entry->GetSite();
    if (siteID != 0 && namebase_) {
        return std::string(namebase_->GetName(NAME_SITE, siteID));
    }
    return "?";
}

std::string PGNConverter::GenerateDateTag(IndexEntry* entry) {
    dateT date = entry->GetDate();
    return FormatDate(date);
}

std::string PGNConverter::GenerateRoundTag(IndexEntry* entry) {
    idNumberT roundID = entry->GetRound();
    if (roundID != 0 && namebase_) {
        return std::string(namebase_->GetName(NAME_ROUND, roundID));
    }
    return "?";
}

std::string PGNConverter::GenerateWhiteTag(IndexEntry* entry) {
    idNumberT whiteID = entry->GetWhite();
    if (whiteID != 0 && namebase_) {
        return std::string(namebase_->GetName(NAME_PLAYER, whiteID));
    }
    return "?";
}

std::string PGNConverter::GenerateBlackTag(IndexEntry* entry) {
    idNumberT blackID = entry->GetBlack();
    if (blackID != 0 && namebase_) {
        return std::string(namebase_->GetName(NAME_PLAYER, blackID));
    }
    return "?";
}

std::string PGNConverter::GenerateResultTag(IndexEntry* entry) {
    return ResultToString(entry->GetResult());
}

std::string PGNConverter::ConvertMoveTree(Game* game) {
    std::ostringstream pgn;
    Position pos;

    // Initialize position
    if (game->GetStartFlag()) {
        pos.ReadFromFEN(game->GetStartPos());
    } else {
        pos.StdStart();
    }

    // Convert main line moves to SAN notation
    // Note: This requires accessing the move tree structure from Game
    // The exact implementation depends on Game's internal structure

    // Placeholder: iterate through main line moves
    // For each move:
    //   1. Get move in algebraic notation
    //   2. Update position
    //   3. Add to PGN with move number

    return pgn.str();
}

std::string PGNConverter::ConvertMoveToSAN(simpleMoveT* move, Position* pos) {
    // Convert simple move to SAN (Standard Algebraic Notation)
    // This is a complex process that requires:
    // 1. Determining piece type
    // 2. Checking for disambiguation needed
    // 3. Checking for capture
    // 4. Handling castling
    // 5. Handling pawn promotions
    // 6. Handling check/mate symbols

    std::ostringstream san;

    // TODO: Implement full SAN conversion
    // This is non-trivial and requires move validation

    return san.str();
}

std::string PGNConverter::SquareToString(squareT sq) {
    std::ostringstream oss;
    char file = 'a' + (sq % 8);
    char rank = '1' + (sq / 8);
    oss << file << rank;
    return oss.str();
}

std::string PGNConverter::PieceToString(pieceT piece) {
    const char* pieces = " PNBRQK";
    return std::string(1, pieces[piece]);
}

std::string PGNConverter::ResultToString(resultT result) {
    switch (result) {
        case RESULT_White: return "1-0";
        case RESULT_Black: return "0-1";
        case RESULT_Draw: return "1/2-1/2";
        default: return "*";
    }
}

std::string PGNConverter::FormatDate(dateT date) {
    if (date == 0) return "????.??.??";

    uint year = date_GetYear(date);
    uint month = date_GetMonth(date);
    uint day = date_GetDay(date);

    std::ostringstream oss;
    oss << std::setfill('0') << std::setw(4) << year << "."
        << std::setfill('0') << std::setw(2) << month << "."
        << std::setfill('0') << std::setw(2) << day;
    return oss.str();
}
```

### 4. `CMakeLists.txt` (Build Configuration)

```cmake
cmake_minimum_required(VERSION 3.15)
project(scidtopgn VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Source directories
set(SCID_SRC_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../scidvspc/src")
set(SRC_DIR "${CMAKE_CURRENT_SOURCE_DIR}/src")
set(INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/include")

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
)

# New source files
set(TOOL_SOURCES
    ${SRC_DIR}/main.cpp
    ${SRC_DIR}/pgn_converter.cpp
    ${SRC_DIR}/pgn_converter.h
)

# Include directories
include_directories(
    ${SCID_SRC_DIR}
    ${SRC_DIR}
    ${INCLUDE_DIR}
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

### 5. `README.md` (Usage Documentation)

```markdown
# scidtopgn - SCID to PGN Converter

A standalone C++ CLI tool that converts SCID chess database files to PGN format.

## Usage

```bash
scidtopgn <database_path>
```

Where `<database_path>` is the base name of the SCID database (without file extensions).

### Example

```bash
scidtopgn mychessgames
```

This reads `mychessgames.si4`, `mychessgames.sn4`, and `mychessgames.sg4` and outputs PGN to stdout.

### Save to File

```bash
scidtopgn mychessgames > output.pgn
```

## Features

- ✅ Reads SCID database files (.si4, .sn4, .sg4)
- ✅ Converts all games to PGN format
- ✅ Includes all PGN tags (Event, Site, Date, Round, White, Black, Result, ELO, ECO)
- ✅ Handles variations, comments, and NAGs
- ✅ Skips deleted games
- ✅ Uses original SCID decoding logic

## Building

```bash
mkdir build
cd build
cmake ..
make
```

## Dependencies

- C++17 or later
- CMake 3.15 or later
- Original SCID source code (in `../scidvspc/src/`)

## License

Uses SCID source code. Original SCID license applies to reused code.
```

---

## Phase 4: Implementation Steps

### Step 1: Set Up Project Structure

```bash
mkdir -p scidtopgn/src scidtopgn/include scidtopgn/build
```

### Step 2: Create Build Configuration

1. Create `CMakeLists.txt` (as shown above)
2. Ensure original SCID source is at `../scidvspc/src/`

### Step 3: Implement Core Files

1. Create `src/main.cpp` - CLI entry point
2. Create `src/pgn_converter.h` - Interface
3. Create `src/pgn_converter.cpp` - Implementation

### Step 4: Implement PGN Conversion Logic

**Key Challenge**: The `pgn_converter.cpp` needs to implement:

#### SAN Notation Conversion (Complex)

Convert internal move representation to Standard Algebraic Notation:

```cpp
std::string ConvertMoveToSAN(simpleMoveT* move, Position* pos) {
    // 1. Identify piece type
    pieceT piece = pos->GetPieceAt(move->from);
    char pieceChar = PieceToString(piece);

    // 2. For pawn moves, check for capture
    if (piece == PAWN) {
        if (move->capturedPiece != EMPTY) {
            // Capture: include source file (e.g., exd5)
            return SquareToString(move->from)[0] + "x" + SquareToString(move->to);
        }
        // Regular move: just destination (e.g., e4)
        return SquareToString(move->to);
    }

    // 3. For piece moves, check for disambiguation
    // Generate all legal moves for that piece type
    // If multiple pieces can move to same square, add disambiguator
    // (file, rank, or both)

    // 4. Add 'x' for captures
    std::string san = pieceChar;
    if (move->capturedPiece != EMPTY) {
        san += 'x';
    }

    // 5. Add destination square
    san += SquareToString(move->to);

    // 6. Handle promotions
    if (move->promotePiece != EMPTY) {
        san += '=';
        san += PieceToString(move->promotePiece);
    }

    // 7. Handle castling
    if (piece == KING) {
        if (move->to - move->from == 2) {
            return "O-O";  // Kingside
        } else if (move->from - move->to == 2) {
            return "O-O-O";  // Queenside
        }
    }

    // 8. Add check/mate symbols
    Position testPos = *pos;
    testPos.DoSimpleMove(move);
    if (testPos.IsMate()) {
        san += '#';
    } else if (testPos.IsChecked()) {
        san += '+';
    }

    return san;
}
```

#### Move Tree Traversal

Handle variations and nested comments:

```cpp
std::string PGNConverter::ConvertMoveTree(Game* game) {
    std::ostringstream pgn;
    int moveNum = 1;
    bool whiteToMove = true;

    // Traverse main line
    // Game's internal structure provides access to move tree
    // This requires understanding Game's internal node structure

    // Pseudocode:
    // Node* currentNode = game->GetRootNode();
    // while (currentNode) {
    //     if (whiteToMove) {
    //         pgn << moveNum << ". ";
    //     }
    //
    //     // Get move
    //     simpleMoveT* move = currentNode->GetMove();
    //     pgn << ConvertMoveToSAN(move, &pos);
    //
    //     // Handle NAGs (Numeric Annotation Glyphs)
    //     for each NAG in node:
    //         pgn << " $" << NAG;
    //
    //     // Handle comments
    //     if (node has comment) {
    //         pgn << " {" << comment << "}";
    //     }
    //
    //     // Handle variations
    //     if (node has variations) {
    //         pgn << " (";
    //         for each variation:
    //             Convert variation recursively
    //         pgn << ")";
    //     }
    //
    //     // Execute move
    //     pos.DoSimpleMove(move);
    //     whiteToMove = !whiteToMove;
    //     currentNode = currentNode->GetNext();
    //
    //     // Add space between moves
    //     if (whiteToMove) {
    //         moveNum++;
    //     }
    //     pgn << " ";
    // }

    return pgn.str();
}
```

### Step 5: Handle Special Cases

1. **Non-Standard Starting Position**:
   - Check `game->GetStartFlag()`
   - If set, output FEN tag and custom start position

2. **Variations and Comments**:
   - Parse recursively with parentheses for variations
   - Braces for comments

3. **NAGs (Numeric Annotation Glyphs)**:
   - Convert numeric codes to symbols (!, ?, !!, etc.)
   - Use `nagtext.h` from original code

4. **Promotions**:
   - Handle under-promotions (N, B, R)
   - Format: `e8=Q` or `e8=N`

### Step 6: Build and Test

```bash
cd scidtopgn/build
cmake ..
make

# Test with sample database
./scidtopgn ../test_database > test_output.pgn
```

---

## Phase 5: Technical Considerations

### Memory Management

- **SCID objects use new/delete**: Must match pattern
- **ByteBuffer**: Managed by original code
- **NameBase**: Contains string allocator, don't free strings

### File Handles

- Open files in read-only mode (`FMODE_ReadOnly`)
- Close files after use
- Handle errors gracefully

### Move Number Formatting

```cpp
// Correct PGN formatting:
// 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6

// Not:
// 1. e4 2. e5 3. Nf3 4. Nc6
```

### Variations in PGN

```
1. e4 e5 2. Nf3 (2. d4) Nc6 3. Bb5
```

### Comments in PGN

```
1. e4 {Best by test} e5 2. Nf3 {Main line} Nc6
```

---

## Phase 6: Optional Enhancements

### Command-Line Options

```bash
scidtopgn [options] <database>

Options:
  -n <number>     Export specific game number
  -r <range>     Export game range (e.g., 1-100)
  -o <file>      Output to file instead of stdout
  -v             Verbose output
  --no-variants  Exclude variations
```

### Performance Optimizations

- Batch read multiple games
- Use memory-mapped files for large databases
- Parallel processing for multi-core systems

### Error Recovery

- Skip corrupted games with warning
- Continue processing after errors
- Report statistics at end

---

## Summary

### Files to Create

| File | Lines Est. | Purpose |
|------|-----------|---------|
| `src/main.cpp` | ~100 | CLI entry point |
| `src/pgn_converter.h` | ~50 | PGN converter interface |
| `src/pgn_converter.cpp` | ~500 | PGN conversion logic |
| `CMakeLists.txt` | ~50 | Build configuration |
| `README.md` | ~50 | Documentation |

### Total New Code: ~750 lines

### Files Reused (No Modification): ~20 files

### Key Implementation Challenges

1. **SAN notation conversion** - Complex move disambiguation
2. **Move tree traversal** - Handling variations and comments
3. **Check/mate detection** - After each move
4. **Promotion handling** - Including under-promotions
5. **Special position handling** - FEN tags, custom start

### Success Criteria

- ✅ Opens SCID database files correctly
- ✅ Decodes all games without errors
- ✅ Outputs valid PGN format
- ✅ Handles variations and comments
- ✅ Preserves all metadata (tags)
- ✅ No modifications to original SCID code
