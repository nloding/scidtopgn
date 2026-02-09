# Plan: Standalone C++ SCID-to-PGN CLI Tool (Self-Contained)

**⚠️ This is the FINAL plan - self-contained in `scid-cpp/` directory**

- Copies required files from original SCID source
- Builds everything in isolated directory
- No external dependencies during build

---

## Overview

Create a standalone C++ CLI tool (`scidtopgn`) that reads SCID database files (.si4, .sn4, .sg4) and outputs PGN format to stdout.

**Key Constraint**: NO modifications to original C++ files. Copy files to new directory, build there.

**Critical Discovery**: The existing `Game` class has full PGN output functionality via `WriteToPGN(TextBuffer*)`!

---

## Phase 1: Directory Structure

```
scidtopgn/                    # Root directory
├── scid-cpp/                # NEW - Self-contained build directory
│   ├── src/                 # Tool source code
│   │   └── main.cpp         # CLI entry point
│   ├── scid/                # Copied SCID source files
│   │   ├── common.h
│   │   ├── error.h
│   │   ├── mfile.cpp
│   │   ├── mfile.h
│   │   ├── bytebuf.cpp
│   │   ├── bytebuf.h
│   │   ├── index.cpp
│   │   ├── index.h
│   │   ├── namebase.cpp
│   │   ├── namebase.h
│   │   ├── game.cpp
│   │   ├── game.h
│   │   ├── gfile.cpp
│   │   ├── gfile.h
│   │   ├── position.cpp
│   │   ├── position.h
│   │   ├── date.cpp
│   │   ├── date.h
│   │   ├── misc.cpp
│   │   ├── misc.h
│   │   ├── stralloc.cpp
│   │   ├── stralloc.h
│   │   ├── textbuf.cpp
│   │   ├── textbuf.h
│   │   ├── movelist.cpp
│   │   ├── movelist.h
│   │   ├── sqmove.h
│   │   ├── sqlist.h
│   │   ├── sqset.h
│   │   └── attacks.h
│   ├── build/               # Build output directory
│   ├── CMakeLists.txt       # Build configuration
│   └── README.md           # Documentation
└── CPP_CLI_PLAN_SELF_CONTAINED.md  # This plan
```

---

## Phase 2: Setup Script

### `setup.sh` - Copy Required Files

```bash
#!/bin/bash

# Setup script for scid-cpp project
# Copies required SCID source files to self-contained directory

set -e  # Exit on error

# Source directory (original SCID code)
SCID_SOURCE_DIR="../scidvspc/scidvspc-main/src"

# Target directory (self-contained)
TARGET_DIR="./scid-cpp"
SCID_DIR="${TARGET_DIR}/scid"

echo "Setting up scid-cpp project..."

# Create directories
mkdir -p "${TARGET_DIR}/src"
mkdir -p "${SCID_DIR}"

# Files to copy (read-only from original source)
SCID_FILES=(
    "common.h"
    "error.h"
    "mfile.cpp"
    "mfile.h"
    "bytebuf.cpp"
    "bytebuf.h"
    "index.cpp"
    "index.h"
    "namebase.cpp"
    "namebase.h"
    "game.cpp"
    "game.h"
    "gfile.cpp"
    "gfile.h"
    "position.cpp"
    "position.h"
    "date.cpp"
    "date.h"
    "misc.cpp"
    "misc.h"
    "stralloc.cpp"
    "stralloc.h"
    "textbuf.cpp"
    "textbuf.h"
    "movelist.cpp"
    "movelist.h"
    "sqmove.h"
    "sqlist.h"
    "sqset.h"
    "attacks.h"
)

# Copy SCID files
echo "Copying SCID source files..."
for file in "${SCID_FILES[@]}"; do
    if [ -f "${SCID_SOURCE_DIR}/${file}" ]; then
        cp "${SCID_SOURCE_DIR}/${file}" "${SCID_DIR}/${file}"
        echo "  ✓ ${file}"
    else
        echo "  ✗ NOT FOUND: ${file}"
        exit 1
    fi
done

echo ""
echo "✓ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Copy main.cpp to ${TARGET_DIR}/src/"
echo "  2. Copy CMakeLists.txt to ${TARGET_DIR}/"
echo "  3. cd ${TARGET_DIR}/build"
echo "  4. cmake .."
echo "  5. make"
```

Make it executable:
```bash
chmod +x setup.sh
```

---

## Phase 3: New Files to Create

### 1. `scid-cpp/src/main.cpp` (Entry Point)

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

### 2. `scid-cpp/CMakeLists.txt` (Build Configuration)

```cmake
cmake_minimum_required(VERSION 3.15)
project(scidtopgn VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Source directories
set(SCID_DIR "${CMAKE_CURRENT_SOURCE_DIR}/scid")
set(SRC_DIR "${CMAKE_CURRENT_SOURCE_DIR}/src")

# SCID source files (copied from original codebase)
set(SCID_SOURCES
    ${SCID_DIR}/common.h
    ${SCID_DIR}/error.h
    ${SCID_DIR}/mfile.cpp
    ${SCID_DIR}/mfile.h
    ${SCID_DIR}/bytebuf.cpp
    ${SCID_DIR}/bytebuf.h
    ${SCID_DIR}/index.cpp
    ${SCID_DIR}/index.h
    ${SCID_DIR}/namebase.cpp
    ${SCID_DIR}/namebase.h
    ${SCID_DIR}/game.cpp
    ${SCID_DIR}/game.h
    ${SCID_DIR}/gfile.cpp
    ${SCID_DIR}/gfile.h
    ${SCID_DIR}/position.cpp
    ${SCID_DIR}/position.h
    ${SCID_DIR}/date.cpp
    ${SCID_DIR}/date.h
    ${SCID_DIR}/misc.cpp
    ${SCID_DIR}/misc.h
    ${SCID_DIR}/stralloc.cpp
    ${SCID_DIR}/stralloc.h
    ${SCID_DIR}/textbuf.cpp
    ${SCID_DIR}/textbuf.h
    ${SCID_DIR}/movelist.cpp
    ${SCID_DIR}/movelist.h
    ${SCID_DIR}/sqmove.h
    ${SCID_DIR}/sqlist.h
    ${SCID_DIR}/sqset.h
    ${SCID_DIR}/attacks.h
)

# Tool source files
set(TOOL_SOURCES
    ${SRC_DIR}/main.cpp
)

# Include directories
include_directories(
    ${SCID_DIR}
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

### 3. `scid-cpp/README.md` (Usage Documentation)

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
8. Re1 h6 9. h3 O-O 10. Nbd2 Be6 11. Bxe6 fxe6 12. b4 Nd7 13. Qb3 Nb6
14. a5 Nc4 15. Nxc4 Bxc4 16. d4 exd4 17. cxd4 Bxd4 18. Nxd4 Qxd4 19. Rd1 Qc5
20. Qxc5 Nxc5 21. Rxe6 Rad8 22. Re7 Kh8 23. Rxa7 Rxd4 24. Rxa6 Nxe4 25. Ra8+ Kg7
26. Rxe8 Rd1+ 27. Kh2 Rd7 28. b5 c6 29. Rc8 Rb5 30. Rxc6 Rb2 31. Kg3 Nxg5
32. fxg5 Rxg5+ 33. Kf4 g5+ 34. Ke3 Rf5 35. Rd8 h5 36. Rd3 Rf1 37. Kf3 Kf6
38. Kg4 h4 39. Rd6+ Ke5 40. Rd8 Ke7 41. Kh5 Rf5 42. Kh4 Rxg5+ 43. Kxg5 Kf7
44. Rd7+ Ke6 45. Rd8 Ke7 46. Kf4 Kf8 47. Ke5 Kg7 48. Kd6 Kf7 49. Rd7+ Ke8
50. Kc7 1-0
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

### From Source

```bash
# 1. Create build directory
cd scid-cpp
mkdir build
cd build

# 2. Configure with CMake
cmake ..

# 3. Build
make

# 4. (Optional) Install to system
sudo make install
```

### Clean Build

```bash
# Remove build directory and rebuild
cd scid-cpp
rm -rf build
mkdir build
cd build
cmake ..
make
```

## Requirements

- C++17 or later
- CMake 3.15 or later
- SCID source files (copied during setup)

## Project Structure

```
scid-cpp/
├── src/              # Tool source code
│   └── main.cpp      # CLI entry point
├── scid/             # Copied SCID source files
│   ├── game.cpp/h    # Game decoding and PGN output
│   ├── index.cpp/h   # Index file handling
│   ├── namebase.cpp/h # Name file handling
│   └── ...          # Other SCID files
├── build/            # Build output (created by CMake)
├── CMakeLists.txt    # Build configuration
└── README.md         # This file
```

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

## Testing

### Test with Sample Database

```bash
# Assuming you have a test SCID database called "testdb"
./scidtopgn testdb > test_output.pgn

# Verify output
head -20 test_output.pgn
```

### Verify PGN Format

You can verify the PGN output with chess engines:

```bash
# Using pgn-extract (if available)
pgn-extract -s test_output.pgn

# Or using Stockfish (to validate games)
stockfish test_output.pgn
```

## Troubleshooting

### "cannot open index file" Error

Ensure the database files exist:
```bash
ls -l mygames.si4 mygames.sn4 mygames.sg4
```

### "cannot read name file" Error

Check that all three files have the same base name.

### Compilation Errors

Ensure you have C++17 support:
```bash
g++ --version  # Should show version 7.0 or later
```

### "undefined reference" Errors

Ensure all SCID files were copied during setup:
```bash
ls scid/*.cpp scid/*.h
```

## License

Uses SCID source code. Original SCID license applies to reused code.

## Acknowledgments

- SCID: Shane's Chess Information Database
- PGN format: Portable Game Notation Standard
```

---

## Phase 4: Build and Test

### Complete Build Process

```bash
# 1. Run setup script (from root directory)
cd /home/nloding/code/scidtopgn
./setup.sh

# Output:
# ✓ Setup complete!
# Next steps:
#   1. Copy main.cpp to ./scid-cpp/src/
#   2. Copy CMakeLists.txt to ./scid-cpp/
#   3. cd ./scid-cpp/build
#   4. cmake ..
#   5. make

# 2. Copy main.cpp to scid-cpp/src/
# (Already done if you have the files from this plan)

# 3. Copy CMakeLists.txt to scid-cpp/
# (Already done if you have the files from this plan)

# 4. Build the project
cd scid-cpp
mkdir build
cd build
cmake ..
make

# Output:
# -- Configuring done
# -- Generating done
# -- Build files have been written to: /path/to/scid-cpp/build
# [  5%] Building CXX object CMakeFiles/scidlib.dir/scid/common.h.o
# [ 10%] Building CXX object CMakeFiles/scidlib.dir/scid/error.h.o
# ...
# [100%] Linking CXX executable scidtopgn

# 5. Test with sample database
./scidtopgn ../../test_database > test_output.pgn

# 6. Verify output
head -30 test_output.pgn
```

### Build Verification

```bash
# Check that executable was created
ls -lh scidtopgn

# Run version (if supported)
./scidtopgn --version

# Test with small database
./scidtopgn ../../tiny_db > tiny.pgn && echo "SUCCESS"
```

---

## Phase 5: Optional Enhancements

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
  -v, --verbose  Verbose output (show progress)
  -h, --help     Show help message
```

### Progress Reporting

```cpp
#include <sys/ioctl.h>
#include <unistd.h>

void reportProgress(gameNumberT current, gameNumberT total) {
    int progress = (current * 100) / total;

    // Simple progress bar
    fprintf(stderr, "\rProgress: [");
    for (int i = 0; i < 50; i++) {
        if (i < progress / 2) {
            fprintf(stderr, "=");
        } else {
            fprintf(stderr, " ");
        }
    }
    fprintf(stderr, "] %3d%% (%d/%d)",
            progress, current, total);
    fflush(stderr);
}
```

### Error Summary

```cpp
struct Stats {
    gameNumberT totalGames;
    gameNumberT successful;
    gameNumberT skippedDeleted;
    gameNumberT errors;
};

Stats stats = {0};
stats.totalGames = numGames;

// At end of main():
std::cerr << "\n\n=== Conversion Summary ===" << std::endl;
std::cerr << "  Total games:   " << stats.totalGames << std::endl;
std::cerr << "  Successful:    " << stats.successful << std::endl;
std::cerr << "  Skipped:      " << stats.skippedDeleted << " (deleted)" << std::endl;
std::cerr << "  Errors:       " << stats.errors << std::endl;
std::cerr << "  Success rate: "
          << (stats.successful * 100 / stats.totalGames) << "%" << std::endl;
```

---

## Phase 6: Technical Notes

### File Organization

```
scid-cpp/
├── src/main.cpp           # NEW: Tool entry point
├── scid/                 # COPIED: Original SCID code
│   ├── common.h          # Core definitions
│   ├── error.h           # Error codes
│   ├── game.cpp/h        # Game decoding & PGN output
│   ├── index.cpp/h       # Index file handling
│   ├── namebase.cpp/h    # Name file handling
│   └── ...              # Other SCID files
├── build/               # GENERATED: Build artifacts
│   ├── CMakeCache.txt
│   ├── Makefile
│   ├── scidlib/         # Object files
│   └── scidtopgn        # Executable
├── CMakeLists.txt        # Build configuration
└── README.md            # Documentation
```

### Why Self-Contained?

**Advantages:**
1. **No external dependencies** - All code in one directory
2. **Portable** - Can zip and share entire `scid-cpp/` folder
3. **Simple build** - Just run `cmake .. && make` in build/
4. **Isolated** - Changes to original SCID don't affect this project
5. **Version control** - Can track which SCID version is used

**Trade-offs:**
1. **Disk space** - Copies ~20 files (duplicate)
2. **Setup step** - Must run setup script once
3. **Updates** - Need to re-copy if SCID code changes

### Memory Management

- **SCID uses new/delete**: Always match pattern
- **TextBuffer**: Manages its own buffer (don't free GetBuffer())
- **Game object**: Can be reused or recreated per game
- **ByteBuffer**: Managed by SCID code

### File Handle Management

```cpp
// Always close files in correct order
delete idx;      // Closes .si4
delete nb;       // Closes .sn4
gf->Close();     // Closes .sg4
delete gf;
```

---

## Summary

### Files to Create

| File | Lines | Purpose |
|------|-------|---------|
| `setup.sh` | ~70 | Setup script to copy SCID files |
| `scid-cpp/src/main.cpp` | ~150 | CLI entry point, game loop |
| `scid-cpp/CMakeLists.txt` | ~70 | Build configuration |
| `scid-cpp/README.md` | ~250 | Documentation |

**Total New Code: ~540 lines**

### Files Copied from Original SCID: 31 files

| Category | Files |
|----------|-------|
| Core | `common.h`, `error.h` |
| Files | `mfile.cpp/h`, `bytebuf.cpp/h` |
| Index | `index.cpp/h` |
| Names | `namebase.cpp/h` |
| Game | `game.cpp/h` |
| GFile | `gfile.cpp/h` |
| Position | `position.cpp/h`, `movelist.cpp/h` |
| Support | `textbuf.cpp/h`, `date.cpp/h`, `misc.cpp/h`, `stralloc.cpp/h` |
| Utils | `attacks.h`, `sqmove.h`, `sqlist.h`, `sqset.h` |

### Key Implementation Details

**Leverages existing SCID code:**
- `Game::Decode()` - Decodes SCID binary format
- `Game::LoadStandardTags()` - Loads metadata
- `Game::WriteToPGN()` - **Outputs PGN format**
- `TextBuffer` - Stores PGN output

**No custom implementation needed for:**
- ❌ PGN tag formatting
- ❌ SAN notation conversion
- ❌ Move tree traversal
- ❌ Variation handling
- ❌ Comment formatting
- ❌ NAG handling

### Build Process

```
1. Run setup.sh (copies 31 SCID files to scid-cpp/scid/)
2. cd scid-cpp/build
3. cmake .. (generates Makefile)
4. make (compiles all code)
5. ./scidtopgn <database> (run tool)
```

### Success Criteria

- ✅ Self-contained in `scid-cpp/` directory
- ✅ Copies required SCID files during setup
- ✅ Builds without external dependencies
- ✅ Opens SCID database files correctly
- ✅ Decodes all games without errors
- ✅ Outputs standard-compliant PGN
- ✅ Handles all PGN features
- ✅ Preserves all metadata
- ✅ No modifications to original SCID code

---

## Comparison: Approaches

| Approach | New Files | Setup | Portability | Build Complexity |
|----------|-----------|-------|-------------|------------------|
| **Self-Contained** (this plan) | 4 files | setup.sh script | Excellent (zip & share) | Simple (cmake && make) |
| **Reference Original** | 3 files | None | Poor (needs ../scidvspc) | Simple |
| **Custom PGN** | 5 files | None | Excellent | Complex (custom code) |

**Winner: Self-Contained approach** ✅
- Best balance of portability and simplicity
- Easy to distribute and use
- Minimal setup required
