# SCID Data Flow: Encoding/Decoding vs Analysis Tools

This document visualizes how SCID's encoding/decoding core components relate to analysis tools like tree generation and crosstables.

## Overview Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                  SCID Binary Files                       │
│  (.si4, .sn4, .sg4 - stored on disk)           │
└─────────────────────────────────────────────────────────────────┘
                          │
                          ▼ READ
┌─────────────────────────────────────────────────────────────────┐
│           CORE ENCODING/DECODING LAYER                  │
│                                                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  │
│  │ index.cpp  │  │namebase.cpp│  │  game.cpp  │  │
│  │            │  │            │  │            │  │
│  │ Read .si4 │  │Read .sn4  │  │ Read .sg4  │  │
│  │ Metadata   │  │ Names      │  │ Moves      │  │
│  └─────┬────┘  └─────┬──────┘  └─────┬──────┘  │
│         │               │                 │             │
│         ▼               ▼                 ▼             │
│  ┌──────────────────────────────────────────────┐       │
│  │        position.cpp                      │       │
│  │                                         │       │
│  │  - Track piece numbers (List[2][16])     │       │
│  │  - Execute moves with capture swap       │       │
│  │  - Validate move legality               │       │
│  │  - Manage board state                │       │
│  └───────────────────┬──────────────────────┘       │
│                      │                                 │
│                      ▼ RECONSTRUCTED                  │
│              ┌───────────────┐                       │
│              │ Game Object   │                       │
│              │ (in memory)  │                       │
│              │               │                       │
│              │ - Players     │                       │
│              │ - Moves       │                       │
│              │ - Result      │                       │
│              │ - Variations  │                       │
│              └───────┬───────┘                       │
│                      │                                 │
└──────────────────────┼─────────────────────────────────┘
                       │
                       │ SHARED DATA ACCESS
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
┌─────────────────────┐    ┌──────────────────────┐
│  Tree Analysis     │    │  Crosstables       │
│  (tree.cpp)       │    │  (crosstab.cpp)    │
│                    │    │                      │
│  ┌──────────────┐  │    │  ┌────────────────┐ │
│  │Input: Games  │  │    │  │Input: Games    │ │
│  └──────┬───────┘  │    │  └──────┬─────────┘ │
│         │           │    │         │            │
│         ▼           │    │         ▼            │
│  ┌──────────────┐  │    │  ┌────────────────┐ │
│  │Process:      │  │    │  │Process:        │ │
│  │- Traverse    │  │    │  │- Extract       │ │
│  │  each game   │  │    │  │  player IDs   │ │
│  │  move by     │  │    │  │- Get results  │ │
│  │  move        │  │    │  │- Pair players │ │
│  │- Count freqs │  │    │  │- Compute stats│ │
│  │- Find ECO   │  │    │  └──────┬─────────┘ │
│  └──────┬───────┘  │    │         │            │
│         │           │    │         ▼            │
│         ▼           │    │  ┌────────────────┐ │
│  ┌──────────────┐  │    │  │Output:         │ │
│  │Output:       │  │    │  │- Crosstable   │ │
│  │- Opening     │  │    │  │  (A vs B)    │ │
│  │  Tree        │  │    │  │- Score matrix  │ │
│  │- Move stats  │  │    │  │- Tournament   │ │
│  │- ECO codes  │  │    │  │  results      │ │
│  └──────────────┘  │    │  └────────────────┘ │
│                    │    │                    │
└────────────────────┘    └────────────────────┘
        │                         │
        ▼                         ▼
┌─────────────────┐    ┌──────────────────┐
│Opening Tree     │    │Crosstable Table  │
│Display/Export │    │Display/Export   │
└─────────────────┘    └──────────────────┘
```

## Component Categories

### 1. Binary File Layer (.si4, .sn4, .sg4)
**Purpose**: Persistent storage of chess database

| File | Extension | Contains | Encoding Method |
|-------|-----------|-----------|------------------|
| Index | .si4 | Game metadata, dates, ratings, offsets | Big-endian packed fields |
| Names | .sn4 | Player/event/site/round names | Front-coding compression |
| Games | .sg4 | Move data, variations, comments | Piece-specific binary encoding |

### 2. Core Encoding/Decoding Layer
**Purpose**: Read binary files → Reconstruct game objects in memory

| File | Primary Role | Key Operations |
|-------|--------------|----------------|
| `index.cpp` | Read .si4 file | Parse header, extract index entries, decode packed dates/IDs |
| `namebase.cpp` | Read .sn4 file | Decode front-coded names, variable-length integers |
| `game.cpp` | Read .sg4 file | Decode piece-specific moves, tags, variations, NAGs |
| `position.cpp` | Track board state | Manage piece numbers, execute moves, validate legality |

**Output**: In-memory `Game` objects with complete move history, annotations, and metadata

### 3. Analysis Tools Layer
**Purpose**: Analyze decoded game data → Generate statistics/summaries

#### Tree Analysis (`tree.cpp`)

**Input**: Decoded `Game` objects

**Processing**:
```
For each game:
  For each position:
    For each move:
      - Increment move count
      - Update result stats (1-0, 0-1, draw)
      - Calculate score for White
      - Look up/assign ECO code
      - Track Elo statistics
```

**Output**: Opening tree statistics

**Example output**:
```
Position after 1.e4 e5 2.Nf3:
  Nc6:  142 games (Score: 520)
  Nf6:   89 games (Score: 495)
  d6:     67 games (Score: 510)
  d5:     23 games (Score: 480)
  
Move Nc6 → ECO: B20 (Sicilian Defense)
```

#### Crosstables (`crosstab.cpp`)

**Input**: Decoded `Game` objects with player IDs and results

**Processing**:
```
For each game:
  Extract White player ID
  Extract Black player ID
  Extract result (1-0, 0-1, 1/2-1/2, *)
  Store in matrix: results[white][black] = result
```

**Output**: Tournament crosstable

**Example output**:
```
            Carlsen  Ding  Giri  Nepo
Carlsen     -      1-0   1-0   0-1
Ding        0-1    -     1/2   1-0
Giri        0-1    1/2   -     1-0
Nepo        1-0    0-1   0-1    -
```

## Data Flow Examples

### Example 1: Opening Tree Generation

```
1. Read 5 games from .si4/.sn4/.sg4 files
   ↓
2. Decode to Game objects (encoding/decoding layer)
   ↓
3. Traverse each game's moves:
   - Game 1: 1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.O-O
   - Game 2: 1.e4 c5 2.Nf3 d6 3.d4 cxd4
   - Game 3: 1.e4 e5 2.Nf3 Nf6 3.Bc4 Bb4
   - Game 4: 1.e4 c6 2.d4 d5 3.Nc3
   ↓
4. Build opening tree at position after 1.e4:
   - e5: 3 games
   - c5: 1 game
   - c6: 1 game
   ↓
5. Calculate statistics for each continuation:
   - e5: Win=2, Loss=1, Score=667, ECO=C20
   - c5: Win=0, Loss=1, Score=333, ECO=B10
   - c6: Win=0, Loss=1, Score=333, ECO=B00
   ↓
6. Output opening tree for display
```

### Example 2: Crosstable Generation

```
1. Read 10 games from .si4/.sn4/.sg4 files
   ↓
2. Decode to Game objects:
   - Game 1: Carlsen vs Ding, 1-0
   - Game 2: Ding vs Carlsen, 0-1
   - Game 3: Carlsen vs Giri, 1-0
   - Game 4: Giri vs Carlsen, 0-1
   - Game 5: Carlsen vs Nepo, 0-1
   - ...
   ↓
3. Extract player IDs and results:
   - Carlsen (ID=42) vs Ding (ID=17): 1-0
   - Ding (ID=17) vs Carlsen (ID=42): 0-1
   - Carlsen (ID=42) vs Giri (ID=33): 1-0
   - Giri (ID=33) vs Carlsen (ID=42): 0-1
   - ...
   ↓
4. Build result matrix:
   results[42][17] = 1-0
   results[17][42] = 0-1
   results[42][33] = 1-0
   results[33][42] = 0-1
   ↓
5. Calculate tournament statistics:
   - Carlsen: 3-1 (75%)
   - Ding: 1-3 (25%)
   - Giri: 1-3 (25%)
   ↓
6. Output formatted crosstable
```

## Key Distinctions

### Encoding/Decoding (Core Layer)
- ✅ **READS** binary SCID files
- ✅ **WRITES** binary SCID files (for encoding)
- ✅ **UNDERSTANDS** SCID format specifications
- ✅ **PRODUCES** in-memory Game objects
- ❌ **NO ANALYSIS** - just parsing

### Analysis Tools (Tree/Crosstable)
- ❌ **DOES NOT READ** binary files directly
- ❌ **DOES NOT WRITE** binary files
- ✅ **CONSUMES** decoded Game objects
- ✅ **PRODUCES** statistics/summaries
- ✅ **FORMATS** output for display (text/HTML/LaTeX)

## File Dependencies

```
index.cpp     ──── common.h
namebase.cpp  ──── common.h, stralloc.h
game.cpp      ──── common.h, position.h, bytebuf.h
position.cpp  ──── common.h, attacks.h

tree.cpp      ──── common.h, game.h, tree.h
crosstab.cpp   ──── common.h, game.h, namebase.h, crosstab.h
```

**Note**: Analysis tools depend on encoding/decoding layer (through `game.h`), but encoding/decoding layer does NOT depend on analysis tools.

## Implementation Implications

### When Implementing a SCID Decoder:
**Must have:**
- ✅ `index.cpp` equivalent (read .si4)
- ✅ `namebase.cpp` equivalent (read .sn4)
- ✅ `game.cpp` equivalent (read .sg4)
- ✅ `position.cpp` equivalent (track pieces/validate moves)

**Optional (for analysis features):**
- 🔹 `tree.cpp` equivalent (opening statistics)
- 🔹 `crosstab.cpp` equivalent (tournament tables)

### When Implementing a SCID Encoder:
**Must have:**
- ✅ `game.cpp` equivalent (encode moves)
- ✅ `position.cpp` equivalent (track pieces)
- ✅ `index.cpp` equivalent (write .si4)
- ✅ `namebase.cpp` equivalent (write .sn4)
- ✅ `gfile.cpp` equivalent (write .sg4)

**Never need:**
- ❌ `tree.cpp` - encoding doesn't generate statistics
- ❌ `crosstab.cpp` - encoding doesn't generate tables

## Summary

1. **Binary Files** → **Encoding/Decoding Layer** → **Game Objects**
2. **Game Objects** → **Analysis Tools** → **Statistics/Tables**
3. Analysis tools are **downstream consumers** of decoded game data
4. Analysis tools **never touch** the binary file format
5. For a minimal SCID decoder, `tree.cpp` and `crosstab.cpp` are **not required**
