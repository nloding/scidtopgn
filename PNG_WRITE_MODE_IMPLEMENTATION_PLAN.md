# PGN→SCID Write Mode Implementation Plan

## Problem Analysis

**Write mode (PGN→SCID) is NOT WORKING** - games are encoded with wrong parameters.

### Root Cause

My scidtopgn/scid-cpp/ game.cpp uses `Game::Encode(&bb, &entry)` which ALWAYS encodes using **STORED games** from a database via `FirstMove->next` path in Encode().

**Why wrong?**
- Original game.cpp (from scidtopgn) has ONE Encode() that encodes current game moves via `FirstMove->next`
- My copied version is ALSO using `FirstMove->next` - it's IDENTICAL to original!

The issue is that both Encode() functions use `FirstMove->next`, which reads stored games from IndexEntry's StoredLine tree.

### Solution Path Forward

**1. Copy scidvspc's Encode() function**  
   - Full implementation with both modes
   - Lines 4866-4917 (4866 lines)
   - Has logic to distinguish modes

**2. Add EncodeCurrentGame() wrapper**
   - In game.h, create new function: `errorT EncodeCurrentGame(ByteBuffer * buf, IndexEntry * ie)`
   - Implementation calls existing Encode() with FirstMove→next path

**3. Modify main.cpp**  
   - For NEW games: `game.EncodeCurrentGame(&bb, &entry)`
   - For existing games: `game.Encode(&bb, &entry)` (as before)

## Implementation Steps

1. Copy scidvspc Encode() to game.cpp as EncodeCurrentGame()
2. Add EncodeCurrentGame() declaration to game.h
3. Update main.cpp to use EncodeCurrentGame() for new games
4. Test with original database (one) to ensure no regression

## Files to Modify

1. `/home/nloding/code/scidtopgn/scid-cpp/scid/game.h` - Add EncodeCurrentGame() declaration
2. `/home/nloding/code/scidtopgn/scid-cpp/scid/game.cpp` - Add EncodeCurrentGame() implementation from scidvspc
3. `/home/nloding/code/scidtopgn/scid-cpp/src/main.cpp` - Update to use EncodeCurrentGame() for new games

## Notes

- Game.cpp Encode() line 49066 is the target
- Copy scidvspc Encode() (lines 4866-4917)
- Need to ensure game's FirstMove is correctly set during PGN parsing
- Original scidtopgn uses FirstMove→next for SCID→SCID encoding
