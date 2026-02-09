# PGN→SCID Write Mode - Investigation Summary

## Problem Identified

### Core Issue: `Game::Encode()` ALWAYS uses stored game encoding

**Location:** scid/scid-cpp/scid/game.cpp - Line 49027

**Signature:**
```cpp
errorT Game::Encode (ByteBuffer * buf, IndexEntry * ie)
```

**Problem:** There's NO way for Encode() to know whether it's encoding:
1. Current game moves (from PGN parsing, stored in FirstMove->next) - **CORRECT**
2. Stored game moves (from SCID database, stored in StoredLine) - **WRONG**

Both paths lead to `encodeVariation()` which calls:
- Line 4926-4926: `encodeVariation(buf, FirstMove->next, ...)`
- Line 4953-4954: `err = encodeVariation(buf, subVar->next, ...)`

**Line 4926:** encodes from `FirstMove->next` ALWAYS (not current game!)

### Why My pgnparse.cpp fails to encode moves

Looking at my scid-cpp (copied from scidvspc):
- Include files are identical
- `MAX_COMMENT_SIZE = 16000` (vs scidvspc's `16000`)
- PGN parse logic is identical
- Encode() call in main.cpp: `game.Encode(&bb, &entry)`

**BUT:** In scidvspc version, Encode() ALWAYS uses `FirstMove->next`, not `CurrentMove->next`!

### Solution Options

**Option 1: Modify `Game::Encode()` to add mode parameter**

```cpp
errorT Game::Encode (ByteBuffer * buf, IndexEntry * ie, bool useStoredMoves = true)
```

Then modify line 49034 to:
```cpp
err = encodeVariation(buf, FirstMove->next, ..., useStoredMoves);
```

**Option 2: Write new `EncodeCurrentGame()` function**

Instead of modifying Encode(), create a new function that:
```cpp
errorT Game::EncodeCurrentGame (ByteBuffer * buf, IndexEntry * ie)
```

Which encodes from `CurrentMove->next` (current game's moves).

**Option 3: Use scidvspc's Encode() directly**

Don't copy pgnparse.cpp - instead:
- Keep my stub (charset.h)
- Use full scidvspc/scidvspc/main/src/pgnparse.cpp
- Copy its Encode() function (line 4866) which is correct

## Files to Modify

1. `/home/nloding/code/scidtopgn/scid-cpp/scid/game.cpp`
   - Line 49027: Add `bool useCurrentMoves = true` parameter
   - Line 4926: Pass to encodeVariation

2. `/home/nloding/code/scidtopgn/scid/scid-cpp/scid/scidvspc/scid/scidvspc-main/src/pgnparse.cpp`
   - Copy over my pgnparse.cpp
   - Use scidvspc's Encode() function

3. `/home/nloding/code/scidtopgn/scid-cpp/src/main.cpp`
   - Add `#include "pgnparse.h"` to replace stub
   - Modify Encode() to use new function

## Expected Behavior After Fix

**Write mode (PGN→SCID):**
1. PgnParser parses PGN
2. Game object populated with moves
3. Call `game.EncodeCurrentGame(buf, &entry)` → encodes from CURRENT moves
4. IndexEntry gets NumHalfMoves → CORRECT value
5. GFile.AddGame() → writes CORRECT game data

**Read mode (SCID→PGN):**
1. IndexEntry->Decode() reads stored game data
2. Game.Decode() populates moves from stored game
3. Call game.EncodeCurrentGame() → encodes from CURRENT moves

## Next Steps

1. Backup current code
2. Copy scidvspc's Encode() to scid-cpp/scid/scid/scidvspc-main/src/pgnparse.cpp
3. Modify game.cpp to add EncodeCurrentGame() function
4. Recompile and test

## Status

- ✅ scidvspc Encode() uses FirstMove→next - CORRECT for reading SCID files
- ✅ My pgnparse.cpp uses FirstMove→next (WRONG for new PGN
- ⚠️ Current game.cpp Encode() uses FirstMove→next (WRONG for new PGN

The issue is that my pgnparse.cpp (from scidvspc) and scid-cpp (from scidvspc) are using different Encode() functions and the Game objects may not be compatible.

## Recommendation

**Copy scidvspc's Encode()** - it's the correct implementation for writing games. It encodes from the game's move list starting from FirstMove→next.

**OR** - Write a wrapper function in Game.cpp that:
- Calls the correct Encode() from game class
- Or uses encodeVariation() directly with proper `FirstMove->next` for move iteration

This will ensure proper encoding for both read and write operations.
