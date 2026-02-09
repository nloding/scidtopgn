# scidt - SCID Database Management Tool Blueprint

**Complete technical specification of the `scidt` command-line utility and its interaction with the SCID database format.**

---

## Overview

`scidt` is the primary command-line interface (CLI) tool for managing SCID chess databases. It provides database inspection, listing, sorting, compaction, and name management operations.

**Location**: `scidvspc/src/scidt.cpp` (774 lines)

**Compilation**: Standard C/C++ compiler with SCID source dependencies

**Usage Pattern**:
```bash
scidt -<option> <database>
```

Where `<database>` is the base name (without extension) of the SCID database files:
- `database.si4` - Index file
- `database.sn4` - Name file
- `database.sg4` - Game file

---

## Command-Line Options Reference

### Option Matrix

| Option | Category | Creates/Updates | Read-Only | Description |
|--------|-----------|------------------|------------|
| `-i` | Info | ❌ No | ✅ Yes | Display general database information |
| `-d` | Metadata | ✅ **YES** | ❌ No | Change database description |
| `-l` | Listing | ❌ No | ✅ Yes | List all games |
| `-c` | Statistics | ❌ No | ✅ Yes | Show compaction statistics |
| `-C` | Compaction | ✅ **YES** | ❌ No | Compact database (remove deleted games) |
| `-n` | Names | ❌ No | ✅ Yes | Display name information |
| `-N` | Compaction | ✅ **YES** | ❌ No | Remove unused names from namebase |
| `-p[prefix]` | Search | ❌ No | ✅ Yes | List players starting with prefix |
| `-e[prefix]` | Search | ❌ No | ✅ Yes | List events starting with prefix |
| `-s[prefix]` | Search | ❌ No | ✅ Yes | List sites starting with prefix |
| `-r[prefix]` | Search | ❌ No | ✅ Yes | List rounds starting with prefix |
| `-S[fields]` | Sorting | ✅ **YES** | ❌ No | Sort database (forward) |
| `-R[fields]` | Sorting | ✅ **YES** | ❌ No | Sort database (reverse) |
| `-D` | Debug | ❌ No | ✅ Yes | Print debug information |

### Sort Fields

Available sort criteria for `-S` and `-R` options:
- `date` - Sort by game date
- `year` - Sort by year
- `event` - Sort by event name
- `site` - Sort by site name
- `round` - Sort by round name
- `white` - Sort by white player name
- `black` - Sort by black player name
- `eco` - Sort by ECO code
- `result` - Sort by game result
- `length` - Sort by game length
- `rating` - Sort by average rating
- `country` - Sort by country

---

## Option 1: `-i` - General Database Information

### Purpose
Display basic information about the SCID database without listing all games.

### Call Chain

```
main()
  └─> usage()                           [Line 56-77]
       └─> Option parsing
       └─> Index * idx = new Index         [Line 591]
              └─> idx->SetFileName("database")
                  └─> Sets base name (adds .si4 automatically)
       └─> NameBase * nb = new NameBase [Line 592]
              └─> nb->SetFileName("database")
                  └─> Sets base name (adds .sn4 automatically)
       └─> Option '-i' selected [Line 681]
              └─> idx->OpenIndexFile(FMODE_ReadOnly)  [scidt.cpp:683]
                    └─> Opens database.si4
                    └─> Reads and validates 182-byte header
                    └─> Checks magic: "Scid.si\0"
                    └─> Reads version: big-endian uint16 (should be 400)
                    └─> Reads game count: big-endian uint24
                    └─> Reads description, custom flags
       └─> nb->ReadNameFile()                [scidt.cpp:685]
                    └─> Opens database.sn4
                    └─> Reads 36-byte header
                    └─> Reads name counts: players, events, sites, rounds
                    └─> Reads front-coded name records
       └─> nb->CloseNameFile()               [scidt.cpp:698]
       └─> Print database information          [scidt.cpp:700-702]
                  └─> Displays:
                      - Database name and description
                      - Game count
                      - Version (Major.Minor format)
                      - Version compatibility warning if applicable
                      - Name counts (players, events, sites, rounds)
```

### File/Class References

| Reference | Location | Purpose |
|-----------|-----------|---------|
| `Index` class | `index.cpp`/`index.h` | Manages .si4 file (index entries) |
| `NameBase` class | `namebase.cpp`/`namebase.h` | Manages .sn4 file (front-coded names) |
| `Index::OpenIndexFile()` | `index.cpp:679` | Opens .si4 file, reads header |
| `Index::GetDescription()` | `index.h:661-662` | Returns database description string |
| `Index::GetNumGames()` | `index.h:706` | Returns total game count |
| `Index::GetVersion()` | `index.h:658-659` | Returns SCID version number |
| `NameBase::ReadNameFile()` | `namebase.cpp` | Opens .sn4 file, reads all names |
| `NameBase::CloseNameFile()` | `namebase.cpp` | Closes .sn4 file |
| `NameBase::GetNumNames(NAME_PLAYER)` | `namebase.h` | Returns player count |
| `NameBase::GetNumNames(NAME_EVENT)` | `namebase.h` | Returns event count |
| `NameBase::GetNumNames(NAME_SITE)` | `namebase.h` | Returns site count |
| `NameBase::GetNumNames(NAME_ROUND)` | `namebase.h` | Returns round count |

### Key Data Structures Accessed

#### From `index.h` (Lines 49-64):
```cpp
struct indexHeaderT {
    char magic[9];              // "Scid.si\0"
    versionT version;           // SCID version (400 = 4.0)
    uint baseType;              // Database type
    gameNumberT numGames;      // Total games (max: 16,777,213)
    gameNumberT autoLoad;       // Game to auto-load (0 = none)
    char description[108];       // Database description
    char customFlagDesc[6][9]; // Custom flag descriptions
};
```

---

## Option 2: `-l` - List All Games

### Purpose
Print a formatted list of all games in the database with selectable fields.

### Call Chain

```
main()
  └─> Option '-l' selected [Line 707]
       └─> Index * idx = new Index         [scidt.cpp:591]
       └─> NameBase * nb = new NameBase         [scidt.cpp:592]
       └─> Initialize objects (same as -i)
       └─> idx->OpenIndexFile(FMODE_ReadOnly) [scidt.cpp:683]
       └─> nb->ReadNameFile()                [scidt.cpp:685]
       └─> PrintGameList(idx, nb, ...)      [scidt.cpp:706-721]
```

### PrintGameList() Function Details

**Location**: `scidt.cpp` Lines 107-121

```cpp
void PrintGameList(Index * i, NameBase * nb,
                    gameNumberT first, gameNumberT last,
                    char * fields, FILE * outf)
{
    char temp[1024];
    gameNumberT gn;
    IndexEntry iE;

    for (gn = first; gn <= last; gn++) {
        // Read each index entry
        i->ReadEntries(&iE, gn-1, 1);

        // Format and print game information
        iE.PrintGameInfo(temp, gn, gn, nb, fields, "");
        fputs(temp, outf);
        putc('\n', outf);
    }
}
```

#### IndexEntry::PrintGameInfo()

**Location**: `index.cpp` - Prints formatted game metadata

**Fields printed** (controlled by `fields` string):
- Game number
- White player name (from NameBase)
- Black player name (from NameBase)
- Event name (from NameBase)
- Site name (from NameBase)
- Round name (from NameBase)
- Date (from index date field)
- Result (1-0, 0-1, 1/2-1/2, *)
- ECO code
- White ELO
- Black ELO
- Game length (half-moves)
- Rating type (Elo, FIDE, etc.)

---

## Option 3: `-c` - Show Compaction Statistics

### Purpose
Display statistics about space efficiency in the database without actually compacting.

### Call Chain

```
main()
  └─> Option '-c' selected [Line 717]
       └─> Initialize objects (same as -i)
       └─> idx->OpenIndexFile(FMODE_ReadOnly)  [scidt.cpp:683]
       └─> nb->ReadNameFile()                [scidt.cpp:685]
       └─> printCompactInfo(idx)               [scidt.cpp:718-730]
```

### printCompactInfo() Function Details

**Location**: `scidt.cpp` Lines 698-730

```cpp
void printCompactInfo(Index * idx)
{
    IndexEntry iE;
    errorT err;

    // Open index file
    err = idx->OpenIndexFile(FMODE_ReadOnly);
    // (Same header reading as -i option)

    // Calculate compaction statistics
    uint nFullBlocks = 0;       // Count blocks that are full
    uint lastBlockBytes = 0;      // Bytes used in last block
    uint gameCount = 0;           // Count non-deleted games
    uint oldBytes = 0;             // Current database size

    // Scan all games
    for (uint i = 0; i < idx->GetNumGames(); i++) {
        idx->ReadEntries(&iE, i, 1);

        if (!iE.GetDeleteFlag()) {
            gameCount++;

            // Check if game fits in current block
            if (lastBlockBytes + iE.GetLength() <= GF_BLOCKSIZE) {
                lastBlockBytes += iE.GetLength();
            } else {
                // Game spans to new block
                nFullBlocks++;
                lastBlockBytes = iE.GetLength();
            }
        }
    }

    // Calculate old size (games + overhead)
    uint newBytes = nFullBlocks * GF_BLOCKSIZE + lastBlockBytes;
    oldBytes = newBytes + (gameCount * INDEX_ENTRY_SIZE) + INDEX_HEADER_SIZE;

    // Get current .sg4 file size
    FILE * pfGfile;
    sprintf(temp, "%s%s", filename, GFILE_SUFFIX);
    pfGfile = fopen(temp, "rb");
    fseek(pfGfile, 0, SEEK_END);
    oldBytes += ftell(pfGfile);
    fclose(pfGfile);

    // Print statistics
    printf("Current size:   %7u games, %8u bytes = %5u Kb\n",
           gameCount, oldBytes, (oldBytes + 512) / 1024);
    printf("Compacted size: %7u games, %8u bytes = %5u Kb\n",
           gameCount, newBytes, (newBytes + 512) / 1024);
}
```

### Constants Used

| Constant | Value | Location | Purpose |
|----------|-------|-----------|---------|
| `GF_BLOCKSIZE` | 131,072 (128KB) | `common.h` | Block size for .sg4 file |
| `INDEX_ENTRY_SIZE` | 47 | `index.h` | Size of each index entry |
| `INDEX_HEADER_SIZE` | 182 | `index.h` | Size of index file header |
| `GFILE_SUFFIX` | ".sg4" | `gfile.h` | Game file extension |
| `INDEX_SUFFIX` | ".si4" | `index.h` | Index file extension |

---

## Option 4: `-C` - Compact Database

### Purpose
Remove games marked for deletion and rewrite both index and game files to eliminate wasted space.

### Call Chain

```
main()
  └─> Option '-C' selected [Line 736]
       └─> compactGameFile()                  [scidt.cpp:746-856]
```

### compactGameFile() Function Details

**Location**: `scidt.cpp` Lines 746-856

```cpp
void compactGameFile()
{
    Index * idxOld, * idxNew;
    GFile * gfOld, * gfNew;
    char tempName[1024];
    errorT err;

    // Create Index and GFile objects for old and new files
    idxOld = new Index;  idxNew = new Index;
    gfOld = new GFile;   gfNew = new GFile;

    // Create backup filenames (database_OLD.si4, database_OLD.sg4)
    strCopy(tempName, filename);
    strAppend(tempName, "_OLD");

    // Rename original files to _OLD (safety mechanism)
    if (renameFile(filename, tempName, INDEX_SUFFIX) != OK ||
        renameFile(filename, tempName, GFILE_SUFFIX) != OK) {
        // Error handling
        exit(1);
    }

    // Set up Index and GFile objects
    idxOld->SetFileName(tempName);
    idxNew->SetFileName(filename);

    // Open old files
    err = idxOld->OpenIndexFile(FMODE_ReadOnly);
    if (err != OK) { fileErr(...); }
    err = gfOld->Open(tempName);
    if (err != OK) { fileErr(...); }

    // Create new files
    err = idxNew->CreateIndexFile(FMODE_WriteOnly);
    if (err != OK) { fileErr(...); }
    err = gfNew->Create(filename, FMODE_WriteOnly);
    if (err != OK) { fileErr(...); }

    // Copy games (excluding deleted ones)
    printf("Compacting database...\n");

    ProgBar progBar(stdout);
    progBar.Start();

    ByteBuffer * bbuf = new ByteBuffer;
    bbuf->SetBufferSize(BBUF_SIZE);  // 32,000 bytes

    gameNumberT newNumGames = 0;
    uint oldNumGames = idxOld->GetNumGames();

    bool treefileOutOfDate = false;

    // Iterate through all games
    for (uint i = 0; i < oldNumGames; i++) {
        progBar.Update(i * 100 / oldNumGames);

        IndexEntry ieOld, ieNew;
        idxOld->ReadEntries(&ieOld, i, 1);

        // Skip deleted games
        if (ieOld.GetDeleteFlag()) {
            treefileOutOfDate = true;  // Mark tree file as out-of-date
            continue;
        }

        // Add game to new index
        err = idxNew->AddGame(&newNumGames, &ieNew);
        if (err != OK) { break; }

        // Copy game data
        ieNew = ieOld;
        bbuf->Empty();
        err = gfOld->ReadGame(bbuf, ieOld.GetOffset(), ieOld.GetLength());
        if (err != OK) { break; }
        bbuf->BackToStart();

        uint offset = 0;
        err = gfNew->AddGame(bbuf, &offset);
        if (err != OK) { break; }
    }

    // Update index and game file types
    idxNew->SetType(idxOld->GetType());

    // Close all files
    idxOld->CloseIndexFile();
    idxNew->CloseIndexFile();
    gfOld->Close();
    gfNew->Close();

    // Remove _OLD files (if successful)
    removeFile(tempName, INDEX_SUFFIX);
    removeFile(tempName, GFILE_SUFFIX);

    // Remove outdated tree file
    if (treefileOutOfDate) {
        removeFile(filename, TREEFILE_SUFFIX);
    }

    progBar.Finish();
    printf("Database was successfully compacted.\n");
}
```

### Key Classes/Methods Used

| Class | Method | Location | Purpose |
|-------|----------|-----------|---------|
| `Index` | `OpenIndexFile()` | `index.cpp:679` | Open .si4 file |
| `Index` | `ReadEntries()` | `index.cpp:688` | Read index entries into memory |
| `Index` | `AddGame()` | `index.cpp:707` | Add entry to index (in-memory) |
| `Index` | `WriteEntries()` | `index.cpp:689` | Write index entries to file |
| `Index` | `CreateIndexFile()` | `index.cpp:682` | Create new .si4 file |
| `Index` | `CloseIndexFile()` | `index.cpp:685` | Close .si4 file |
| `Index` | `GetNumGames()` | `index.h:706` | Get total game count |
| `Index` | `GetType()` | `index.h:656` | Get database type |
| `GFile` | `Open()` | `gfile.cpp` | Open .sg4 file |
| `GFile` | `ReadGame()` | `gfile.cpp:240` | Read game data into buffer |
| `GFile` | `AddGame()` | `gfile.cpp:259` | Write game data to file |
| `GFile` | `Create()` | `gfile.cpp` | Create new .sg4 file |
| `GFile` | `Close()` | `gfile.cpp:70` | Close .sg4 file |
| `ByteBuffer` | `Empty()` | `bytebuf.cpp` | Clear buffer |
| `ByteBuffer` | `BackToStart()` | `bytebuf.cpp` | Reset read position |
| `IndexEntry` | `GetDeleteFlag()` | `index.h:300-307` | Check if game is deleted |
| `IndexEntry` | `GetOffset()` | `index.h:66` | Get game offset in .sg4 |
| `IndexEntry` | `GetLength()` | `index.h:263-264` | Get game data length |

---

## Option 5: `-n` - Name Information

### Purpose
Display statistics about names stored in the namebase (players, events, sites, rounds).

### Call Chain

```
main()
  └─> Option '-n' selected [Line 659]
       └─> Initialize Index and NameBase objects [scidt.cpp:591-594]
       └─> idx->OpenIndexFile(FMODE_ReadOnly) [scidt.cpp:683]
       └─> nb->ReadNameFile()                [scidt.cpp:685]
       └─> printNameInfo(nb, idx)             [scidt.cpp:724-787]
```

### printNameInfo() Function Details

**Location**: `scidt.cpp` Lines 724-787

```cpp
void printNameInfo(NameBase * nb, Index * idx)
{
    errorT err;
    nameT nt;
    const char *ntStr[4] = {"PLAYER", "EVENT", "SITE", "ROUND"};

    // Open files
    err = nb->ReadNameFile();  // Opens .sn4 file
    if (err != OK) { fileErr(...); }

    err = idx->OpenIndexFile(FMODE_ReadOnly);  // Opens .si4 file
    if (err != OK) { fileErr(...); }

    // Recalculate name frequencies by scanning all index entries
    recalcNameFrequencies(nb, idx);

    // Print statistics for each name type
    printf("Database %s: %d players, %d events, %d sites, %d rounds.\n",
           filename,
           nb->GetNumNames(NAME_PLAYER),
           nb->GetNumNames(NAME_EVENT),
           nb->GetNumNames(NAME_SITE),
           nb->GetNumNames(NAME_ROUND));

    // Print detailed info for each name type
    for (nt = NAME_PLAYER; nt <= NAME_LAST; nt++) {
        printNameTypeInfo(nb, idx, nt, ntStr, "players");
    }
}
```

### printNameTypeInfo() Function

**Location**: `scidt.cpp` Lines 789-821

```cpp
void printNameTypeInfo(NameBase * nb, Index * idx,
                       nameT nt, const char *ntStr, const char *typeName)
{
    uint numNames = nb->GetNumNames(nt);
    uint prefix = 0;
    uint length = 0;
    idNumberT currentID = 0, prevID = 0;
    uint longest = 0;
    idNumberT longestID = 0;
    idNumberT mostFrequentID = nb->GetMostFrequent(nt);
    uint mostFrequent = nb->GetFrequency(nt, mostFrequentID);
    uint numUnused = 0;

    // Iterate through all names
    nb->IterateStart(nt);
    for (uint i = 0; i < numNames; i++) {
        prevID = currentID;
        nb->Iterate(nt, &currentID);

        // Calculate statistics
        uint len = strLength(nb->GetName(nt, currentID));
        length += len;

        if (nb->GetFrequency(nt, currentID) == 0) {
            numUnused++;
        }

        if (len > longest) {
            longest = len;
            longestID = i;
        }

        // Calculate prefix similarity (for display)
        if (i > 0) {
            const char *s1 = nb->GetName(nt, currentID);
            const char *s2 = nb->GetName(nt, prevID);
            prefix += strPrefix(s1, s2);
        }
    }

    // Print results
    printf("\n%s section:    %d names, %d unused, %d disk bytes\n",
           ntStr, numNames, numUnused, nb->GetNumBytes(nt));
    printf("Avg length = %.2f bytes, avg prefix = %.2f bytes\n",
           (float) length / numNames,
           (float) prefix / numNames);
    printf("Longest name: \"%s\" (%u bytes)\n",
           nb->GetName(nt, longestID), longest);
    printf("Most frequent name: \"%s\" (%u times)\n",
           nb->GetName(nt, mostFrequentID), mostFrequent);

    // List names matching prefix if provided
    uint matches = nb->DumpAllNames(nt, prefixStr, stdout);
    printf("[%d names matched supplied prefix string]\n", matches);
}
```

### NameBase Methods Used

| Method | Location | Purpose |
|---------|-----------|---------|
| `NameBase::ReadNameFile()` | `namebase.cpp` | Opens .sn4 file, reads header and all names |
| `NameBase::CloseNameFile()` | `namebase.cpp` | Closes .sn4 file |
| `NameBase::GetNumNames(nt)` | `namebase.cpp` | Get count of specified name type |
| `NameBase::GetName(nt, id)` | `namebase.cpp` | Get name string by ID and type |
| `NameBase::GetFrequency(nt, id)` | `namebase.cpp` | Get frequency of name |
| `NameBase::GetMostFrequent(nt)` | `namebase.cpp` | Get most frequently used name ID |
| `NameBase::ZeroAllFrequencies(nt)` | `namebase.cpp` | Reset all frequencies to 0 |
| `NameBase::IncFrequency(nt, id)` | `namebase.cpp` | Increment frequency counter |
| `NameBase::IterateStart(nt)` | `namebase.cpp` | Start iteration over names |
| `NameBase::Iterate(nt, &id)` | `namebase.cpp` | Get next name ID in iteration |
| `NameBase::DumpAllNames(nt, prefix, out)` | `namebase.cpp` | List all names matching prefix |

---

## Option 6: `-N` - Remove Unused Names

### Purpose
Compact the namebase by removing names that are no longer referenced by any game in the database.

### Call Chain

```
main()
  └─> Option '-N' selected [Line 659]
       └─> compactNameBase(nb)                [scidt.cpp:89-419]
```

### compactNameBase() Function Details

**Location**: `scidt.cpp` Lines 895-923

```cpp
void compactNameBase(NameBase * nb)
{
    errorT err;

    // 1. Read original namebase
    err = nb->ReadNameFile();
    if (err != OK) { fileErr(...); }

    // 2. Create temporary index to scan games
    Index * idxTemp = new Index;
    idxTemp->SetFileName(filename);
    err = idxTemp->OpenIndexFile(FMODE_ReadOnly);
    if (err != OK) { fileErr(...); }

    // 3. Recalculate name frequencies by scanning all games
    recalcNameFrequencies(nb, idxTemp);
    idxTemp->CloseIndexFile();

    // 4. Create new namebase with only used names
    NameBase * nbNew = new NameBase;

    // Build ID mapping from old to new IDs
    idNumberT idMapping[NUM_NAME_TYPES];
    for (nameT nt = NAME_FIRST; nt <= NAME_LAST; nt++) {
        idMapping[nt] = new idNumberT[nb->GetNumNames(nt)];
        idNumberT numNames = nb->GetNumNames(nt);
        for (idNumberT oldID = 0; oldID < numNames; oldID++) {
            const char * name = nb->GetName(nt, oldID);
            uint frequency = nb->GetFrequency(nt, oldID);

            // Only add names that are still referenced
            if (frequency > 0) {
                uint newID;
                err = nbNew->AddName(nt, name, &newID);
                if (err != OK) {
                    printf("Error compacting namebase! Aborting...\n");
                    exit(1);
                }
                nbNew->IncFrequency(nt, newID, frequency);
                idMapping[nt][oldID] = newID;
            } else {
                idMapping[nt][oldID] = 0;  // Mark unused
            }
        }
    }

    // 5. Create backup and new files
    Index * idxOld = new Index;
    Index * idxNew = new Index;
    GFile * gfOld = new GFile;
    GFile * gfNew = new GFile;
    char tempName[1024];

    strCopy(tempName, filename);
    strAppend(tempName, "_OLD");

    // Rename: database.si4 -> database_OLD.si4
    // Rename: database.sn4 -> database_OLD.sn4
    // Create: database.si4, database.sn4
    // Create: database.sg4
    if (rename errors occur, abort)
    // ... (same safety pattern as -C option)

    // 6. Update old index entries with new name IDs
    idxOld->SetFileName(tempName);
    idxNew->SetFileName(filename);
    idxNew->SetType(idxOld->GetType());

    err = idxOld->OpenIndexFile(FMODE_ReadOnly);
    // ... open _OLD files ...

    err = idxNew->CreateIndexFile(FMODE_WriteOnly);
    // ... create new files ...

    // 7. Copy each game entry and update name IDs
    gameNumberT newNumGames = 0;
    uint oldNumGames = idxOld->GetNumGames();

    for (uint i = 0; i < oldNumGames; i++) {
        IndexEntry ieOld, ieNew;
        idxOld->ReadEntries(&ieOld, i, 1);

        // Verify game entry against namebase
        if (ieOld.Verify(nb) != OK) {
            fprintf(stderr, "Warning: game %u: ", i+1);
            fprintf(stderr, "names were corrupt, they may be incorrect.\n");
        }

        // Add game to new index
        err = idxNew->AddGame(&newNumGames, &ieNew);
        if (err != OK) { break; }

        // Set name IDs from mapping
        ieNew = ieOld;
        ieNew.SetWhite(idMapping[NAME_PLAYER][ieOld.GetWhite()]);
        ieNew.SetBlack(idMapping[NAME_PLAYER][ieOld.GetBlack()]);
        ieNew.SetEvent(idMapping[NAME_EVENT][ieOld.GetEvent()]);
        ieNew.SetSite(idMapping[NAME_SITE][ieOld.GetSite()]);
        ieNew.SetRound(idMapping[NAME_ROUND][ieOld.GetRound()]);
    }

    // 8. Write new index entries
    printf("Writing new name and index files...\n");

    idxNew->ReadEntireFile();
    err = idxNew->WriteEntries(&ieNew, newNumGames, 1);
    if (err != OK) {
        fprintf(stderr, "ERROR: I/O error!\n");
        exit(1);
    }

    // 9. Write new namebase
    err = nbNew->WriteNameFile();
    if (err != OK) {
        fprintf(stderr, "ERROR: I/O error!\n");
        exit(1);
    }

    // 10. Cleanup
    idxOld->CloseIndexFile();
    idxNew->CloseIndexFile();
    nbNew->SetFileName(filename);

    // Remove _OLD files (success)
    removeFile(tempName, INDEX_SUFFIX);
    removeFile(tempName, NAMEBASE_SUFFIX);

    printf("New index file successfully created; old index removed.\n");
}
```

### Key IndexEntry Methods

| Method | Location | Purpose |
|---------|-----------|---------|
| `IndexEntry::SetWhite(id)` | `index.h:361` | Set white player ID |
| `IndexEntry::SetBlack(id)` | `index.h:362` | Set black player ID |
| `IndexEntry::SetEvent(id)` | `index.h:363` | Set event ID |
| `IndexEntry::SetSite(id)` | `index.h:364` | Set site ID |
| `IndexEntry::SetRound(id)` | `index.h:365` | Set round ID |
| `IndexEntry::GetWhite()` | `index.h:68-69` | Get white player ID |
| `IndexEntry::GetBlack()` | `index.h:70` | Get black player ID |
| `IndexEntry::GetEvent()` | `index.h:70` | Get event ID |
| `IndexEntry::GetSite()` | `index.h:71` | Get site ID |
| `IndexEntry::GetRound()` | `index.h:72` | Get round ID |
| `IndexEntry::Verify(nb)` | `index.h:59` | Verify entry against namebase |

---

## Option 7: Prefix Search Options (`-p`, `-e`, `-s`, `-r`)

### Purpose
List names (players/events/sites/rounds) that start with a specified prefix string.

### Call Chain

```
main()
  └─> Option '-p'/'-e'/'-s'/'-r' selected [scidt.cpp:597-649]
       └─> Extract prefix string from option[2]
       └─> Set name type (PLAYER/EVENT/SITE/ROUND)
       └─> Initialize Index and NameBase [scidt.cpp:591-594]
       └─> idx->OpenIndexFile(FMODE_ReadOnly) [scidt.cpp:683]
       └─> nb->ReadNameFile()                [scidt.cpp:685]
       └─> List names with prefix matching
}
```

### Implementation Pattern

```cpp
// From scidt.cpp Lines 597-648
char *option = argv[1];
char *prefixStr = &(option[2]);  // Extract prefix (e.g., "Carl")
nameT nt = NAME_PLAYER;  // Set type
const char *ntStr = "players";  // Type name

// Open and iterate names
nb->IterateStart(nt);
for (uint i = 0; i < nb->GetNumNames(nt); i++) {
    prevID = currentID;
    nb->Iterate(nt, &currentID);

    // Check if name starts with prefix
    const char *name = nb->GetName(nt, currentID);
    if (strPrefix(name, prefixStr)) {
        // Print name with ID and frequency
        printf("...");  // Name matches
    }
}

uint matches = nb->DumpAllNames(nt, prefixStr, stdout);
printf("[%d names matched supplied prefix string]\n", matches);
```

---

## Option 8: `-S` / `-R` - Sort Database

### Purpose
Reorder games in the database by specified sort criteria (forward or reverse) and rewrite the index file.

### Call Chain

```
main()
  └─> Option '-S'/'-R' selected [scidt.cpp:721-722]
       └─> Initialize Index and NameBase [scidt.cpp:591-594]
       └─> If sort fields empty, print usage and exit
       └─> Parse sort criteria from option[2]
       └─> Set sort order if '-R' (reverse sort)
       └─> idx->OpenIndexFile(FMODE_ReadOnly) [scidt.cpp:683]
       └─> nb->ReadNameFile()                [scidt.cpp:685]
       └─> idx->ParseSortCriteria(...)      [scidt.cpp:736]
       └─> idx->ReadEntireFile()            [scidt.cpp:695]
       └─> idx->Sort(nb, ...)                [scidt.cpp:714]
       └─> idx->WriteSorted()                [scidt.cpp:722]
       └─> idx->CloseIndexFile()            [scidt.cpp:685]
       └─> removeFile(filename, TREEFILE_SUFFIX)  [scidt.cpp:753]
       └─> Print completion message
}
```

### Sorting Process Details

**Location**: `scidt.cpp` Lines 721-756

```cpp
// Parse sort criteria string (e.g., "date,event,round")
err = idx->ParseSortCriteria(&(option[2]));
if (err != OK) {
    printf("%s\n", idx->ErrorMessage());
    exit(1);
}

// Read all index entries into memory
idx->ReadEntireFile();

// Sort entries by criteria
// Uses heap sort for efficient large-scale sorting
idx->Sort(nb, 1000, &(updateProgress), NULL);

// Write sorted entries back to index file
idx->WriteSorted();

// Remove outdated tree file
// (Tree file depends on game order in index)
removeFile(filename, TREEFILE_SUFFIX);
```

### Index::Sort() Method

**Location**: `index.cpp` Lines 714-727

```cpp
errorT Index::Sort(NameBase * nb,
                    int reportFrequency,
                    void (*progressFn)(void * data, uint progress, uint total),
                    void * progressData)
{
    // Uses in-memory heap sort
    // Sorts all games by specified criteria
    // Updates progress every 1000 games
}
```

### Sort Criteria Parsing

**Location**: `index.cpp` - ParseSortCriteria()

**Supported fields**: `date`, `year`, `event`, `site`, `round`, `white`, `black`, `eco`, `result`, `length`, `rating`, `country`

---

## Option 9: `-d` - Change Database Description

### Purpose
Update the database description stored in the .si4 file header.

### Call Chain

```
main()
  └─> Option '-d' selected [scidt.cpp:663-680]
       └─> Initialize Index object [scidt.cpp:591]
       └─> idx->OpenIndexFile(FMODE_Both)    [scidt.cpp:679]
       └─> Read current header
       └─> idx->SetDescription(newDesc)       [scidt.cpp:661-662]
       └─> idx->WriteHeader()                [scidt.cpp:684]
       └─> idx->CloseIndexFile()            [scidt.cpp:685]
       └─> Print confirmation
}
```

### Implementation

**Location**: `scidt.cpp` Lines 663-680

```cpp
err = idx->OpenIndexFile(FMODE_Both);  // Read-write mode
if (err != OK) { fileErr(filename, INDEX_SUFFIX, err); }

// Read current description
printf("Database \"%s\": %s\n", filename, idx->GetDescription());

// Prompt for new description
char newDesc[1024];
printf("\nEnter a new description: ");
fgets(newDesc, 1024, stdin);  // Read from stdin

// Remove trailing newline
newDesc[strlen(newDesc) - 1] = 0;

// Update header with new description
idx->SetDescription(newDesc);
idx->WriteHeader();  // Rewrite header
idx->CloseIndexFile();

printf("Description changed to: %s\n", idx->GetDescription());
```

### Index Header Structure

```cpp
// From index.h Lines 49-64
struct indexHeaderT {
    char magic[9];                    // "Scid.si\0"
    versionT version;                // Version number
    uint baseType;
    gameNumberT numGames;           // Total games
    gameNumberT autoLoad;            // Auto-load game number
    char description[108];           // Database description (up to 107 chars)
    char customFlagDesc[6][9];     // 6 custom flag descriptions
};
```

---

## Option 10: `-D` - Print Debug Information

### Purpose
Display sizes and structure information for all SCID classes and data structures (for development/debugging).

### Call Chain

```
main()
  └─> Option '-D' selected [scidt.cpp:717]
       └─> printDebugInfo()                     [scidt.cpp:728-761]
}
```

### printDebugInfo() Function

**Location**: `scidt.cpp` Lines 728-761

```cpp
void printDebugInfo()
{
    // Print sizes of all key classes and structures

    printf("Sizes of classes and structs, in bytes:\n");
    printf("\nIndex class: %u\n", (unsigned) sizeof(Index));
    printf("  indexHeaderT struct: %u\n", (unsigned) sizeof(indexHeaderT));
    printf("  IndexEntry class: %u\n", (unsigned) sizeof(IndexEntry));
    printf("  nameNodeT struct: %u\n", (unsigned) sizeof(nameNodeT));
    printf("\nGame class: %u\n", (unsigned) sizeof(Game));
    printf("  patternT struct: %u\n", (unsigned) sizeof(patternT));
    printf("  moveT struct: %u\n", (unsigned) sizeof(moveT));
    printf("  moveChunkT struct: %u\n", (unsigned) sizeof(moveChunkT));
    printf("  tagT struct: %u\n", (unsigned) sizeof(tagT));
    printf("\nGFile class: %u\n", (unsigned) sizeof(GFile));
    printf("  gfBlockT struct: %u\n", (unsigned) sizeof(gfBlockT));

    printf("\nNameBase class: %u\n", (unsigned) sizeof(NameBase));
    printf("  nameBaseHeaderT struct: %u\n", (unsigned) sizeof(nameBaseHeaderT));
    printf("  nameNodeT struct: %u\n", (unsigned) sizeof(nameNodeT));

    printf("\nPosition class: %u\n", (unsigned) sizeof(Position));
    printf("  simpleMoveT struct: %u\n", (unsigned) sizeof(simpleMoveT));
    printf("  LegalMoveList class: %u\n", (unsigned) sizeof(MoveList));
    printf("  sanListT struct: %u\n", (unsigned) sizeof(sanListT));
    printf("   pseudoLegalListT struct: %u\n", (unsigned) sizeof(pseudoLegalListT));

    printf("\nTreeCache class: %u\n", (unsigned) sizeof(TreeCache));
    printf("  cachedTreeT struct: %u\n", (unsigned) sizeof(cachedTreeT));
    printf("  treeT struct: %u\n", (unsigned) sizeof(treeT));
    printf("  treeNodeT struct: %u\n", (unsigned) sizeof(treeNodeT));
}
```

---

## Global Variables and Utilities

### Global State

```cpp
// From scidt.cpp Lines 36-50
char * progname;             // argv[0] - program name
char * filename = NULL;     // argv[2] - database base name
errorT err;                // Error code tracking
char displayStr[] = "g6: w13 W4 b13 B4 r3:m2 y4 s11 o4";

// Progress bar (shared across all operations)
ProgBar * global_ProgBar = new ProgBar(stdout);
```

### Progress Update Callback

```cpp
// From scidt.cpp Lines 45-50
void updateProgress(void * pdata, uint count, uint total)
{
    // Updates global progress bar
    global_ProgBar->Update(count * 100 / total);
}
```

### Error Handling

```cpp
// From scidt.cpp Lines 80-99
void fileErr(const char * fname, const char * suffix, errorT err)
{
    if (err == ERROR_FileOpen) {
        fprintf(stderr, "%s: ERROR: could not open file \"%s%s\"\n",
                progname, fname, suffix);
        fprintf(stderr, "    The file may not exist, or may be read-only.\n");
    } else if (err == ERROR_Corrupt) {
        fprintf(stderr, "%s: ERROR: corrupt data reading \"%s%s\"\n",
                progname, fname, suffix);
    } else {
        fprintf(stderr, "%s: ERROR reading file \"%s%s\"\n",
                progname, fname, suffix);
    }
    exit(1);
}
```

---

## Include File Dependencies

```cpp
// From scidt.cpp Lines 15-25
#include "common.h"       // Core types and constants
#include "index.h"        // Index file management
#include "namebase.h"     // Name file management
#include "misc.h"         // Miscellaneous utilities
#include "date.h"         // Date handling
#include "game.h"         // Game structures
#include "gfile.h"        // Game file management
#include "bytebuf.h"     // Byte buffer operations
#include "textbuf.h"     // Text buffer operations
#include "tree.h"         // Tree/cache management
#include "progbar.h"      // Progress bar display

#include <ctype.h>        // Character type functions
#include <stdio.h>        // Standard I/O
#include <string.h>       // String manipulation
#include <strings.h>       // BSD string functions (non-Windows)
```

---

## Data Flow Summary

### Read-Only Operations (`-i`, `-l`, `-c`, `-n`, prefix searches)

```
┌─────────────────────────────────────────────────────┐
│                 User Command Line                    │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│            scidt main()                    │
│  - Parse arguments                              │
│  - Select option handler                       │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│         Open Files (.si4, .sn4)          │
│  - Index::OpenIndexFile(READ_ONLY)         │
│  - NameBase::ReadNameFile()                │
│  - Files NOT modified                          │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│         Read and Display                      │
│  - Iterate through index entries              │
│  - Lookup names from NameBase                │
│  - Print formatted output                      │
│  - Files remain unchanged                     │
└─────────────────────────────────────────────────────┘
```

### Write Operations (`-d`, `-C`, `-N`, `-S`, `-R`)

```
┌─────────────────────────────────────────────────────┐
│            scidt main()                    │
│  - Parse arguments                              │
│  - Select write option handler                │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│    Create Backup Files (_OLD)                │
│    - Rename original → _OLD                   │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│    Create New Files                          │
│    - Index::CreateIndexFile(WRITE)         │
│    - GFile::Create(WRITE)                 │
│    - Files created empty                         │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│    Process and Copy                         │
│    - Read from _OLD files                   │
│    - Apply filter/transform logic             │
│    - Write to NEW files                     │
│    - Update internal structures                 │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│    Cleanup and Atomic Swap                     │
│    - Write headers to new files               │
│    - Remove _OLD files                      │
│    - Original filenames restored                   │
└─────────────────────────────────────────────────────┘
```

---

## Critical Implementation Patterns

### 1. File Safety with Backup

All write operations use a backup pattern:

1. **Rename original** → `_OLD` (e.g., `database.si4` → `database_OLD.si4`)
2. **Create new** with original name (e.g., `database.si4`)
3. **Process data** (read from `_OLD`, write to new)
4. **Write new headers** (commit operation)
5. **On success**: Remove `_OLD` files
6. **On failure**: Restore from `_OLD`, abort

**Why this pattern?**
- Atomic operation (either succeeds completely or not at all)
- No risk of data corruption if process crashes
- Can recover if something goes wrong

### 2. Index Entry Manipulation

Index entries are manipulated through setter methods:

```cpp
// Get values
idNumberT white = ie.GetWhite();
idNumberT black = ie.GetBlack();
uint offset = ie.GetOffset();
uint length = ie.GetLength();
resultT result = ie.GetResult();
bool deleted = ie.GetDeleteFlag();

// Set values
ieNew.SetWhite(id);
ieNew.SetBlack(id);
ieNew.SetOffset(offset);
ieNew.SetLength(length);
ieNew.SetResult(result);
ieNew.SetDeleteFlag(false);  // Clear delete flag
```

### 3. Progress Bar Integration

Long-running operations use progress callbacks:

```cpp
ProgBar progBar(stdout);
progBar.Start();
// ... process items ...
progBar.Update(count * 100 / total);
// ... more items ...
progBar.Finish();
```

### 4. Error Handling Pattern

Consistent error handling across all operations:

```cpp
err = someFunction(...);
if (err != OK) {
    fileErr(filename, suffix, err);  // Prints and exits
}
```

---

## Integration Points

### For Building a SCID Reader

To read SCID databases, follow this pattern:

```cpp
// 1. Include necessary headers
#include "index.h"
#include "namebase.h"
#include "gfile.h"

// 2. Initialize
Index * idx = new Index;
NameBase * nb = new NameBase;
GFile * gf = new GFile;

// 3. Open files
idx->SetFileName("database");
nb->SetFileName("database");
gf->Open("database");

// 4. Open index and name files
idx->OpenIndexFile(FMODE_ReadOnly);
nb->ReadNameFile();

// 5. Access game data
uint numGames = idx->GetNumGames();
for (uint i = 0; i < numGames; i++) {
    // Get index entry
    IndexEntry * entry = idx->FetchEntry(i);

    // Get metadata
    const char * white = entry->GetWhiteName(nb);
    const char * black = entry->GetBlackName(nb);
    uint offset = entry->GetOffset();
    uint length = entry->GetLength();

    // Read game data
    ByteBuffer bb;
    gf->ReadGame(&bb, offset, length);

    // Decode game (using game.cpp functions)
    Game game;
    game.Decode(&bb, GAME_DECODE_ALL);

    // Access game data through game object
    // (move tree, variations, comments, etc.)
}

// 6. Cleanup
delete idx;
delete nb;
gf->Close();
```

### For Building a SCID Writer

To write SCID databases, follow patterns from `-C`, `-N`, `-S`:

```cpp
// 1. Create objects
Index * idx = new Index;
NameBase * nb = new NameBase;
GFile * gf = new GFile;

// 2. Create new database
idx->SetFileName("database");
nb->SetFileName("database");
gf->Open("database");

// 3. Add games
for each game to add {
    IndexEntry entry;
    entry.SetWhite(whiteID);
    entry.SetBlack(blackID);
    entry.SetOffset(offset);
    entry.SetLength(length);
    entry.SetResult(result);
    // ... set all other fields ...

    idx->AddGame(&gameNum, &entry);
}

// 4. Write namebase
nb->AddName(NAME_PLAYER, playerName, &playerID);
nb->WriteNameFile();
```

---

## Constants and File Formats

### Index File (.si4)

| Component | Offset | Size | Description |
|----------|--------|------|-------------|
| Magic | 0-7 | 8 bytes | "Scid.si\0" |
| Version | 8-9 | 2 bytes | Big-endian uint16 (400 = 4.0) |
| Base Type | 10-13 | 4 bytes | Database type |
| Game Count | 14-16 | 3 bytes | Big-endian uint24 (max: 16,777,213) |
| Auto Load | 17-19 | 3 bytes | Big-endian uint24 |
| Description | 20-127 | 108 bytes | ASCII text (null-terminated) |
| Custom Flag Desc | 128-177 | 54 bytes | 6 × 9-byte descriptions |
| **Header Total** | 0-181 | 182 bytes | |
| **Entry** | 182+ (n×47) | 47 bytes | Variable per game |

### Name File (.sn4)

| Component | Offset | Size | Description |
|----------|--------|------|-------------|
| Magic | 0-7 | 8 bytes | "Scid.sn\0" |
| Timestamp | 8-11 | 4 bytes | Big-endian uint32 |
| Player Count | 12-14 | 3 bytes | Big-endian uint24 |
| Event Count | 15-17 | 3 bytes | Big-endian uint24 |
| Site Count | 18-20 | 3 bytes | Big-endian uint24 |
| Round Count | 21-23 | 3 bytes | Big-endian uint24 |
| Max Player Freq | 24-26 | 3 bytes | Big-endian uint24 |
| Max Event Freq | 27-29 | 3 bytes | Big-endian uint24 |
| Max Site Freq | 30-32 | 3 bytes | Big-endian uint24 |
| Max Round Freq | 33-35 | 3 bytes | Big-endian uint24 |
| **Header Total** | 0-35 | 36 bytes | |
| **Data** | 36-EOF | Variable | Front-coded names |

### Game File (.sg4)

| Component | Size | Description |
|----------|------|-------------|
| Block Size | - | 131,072 bytes (128KB) |
| Game Data | Variable | Encoded moves, variations, comments |
| Organization | - | Block-based for efficient access |
| Max Game Size | - | 131,071 bytes (2^17 - 1) |

---

## Summary

**`scidt`** is a comprehensive database management CLI tool that provides:

### Read-Only Operations:
- ✅ Database information (`-i`)
- ✅ Game listing (`-l`)
- ✅ Compaction statistics (`-c`)
- ✅ Name information (`-n`)
- ✅ Prefix-based name search (`-p`, `-e`, `-s`, `-r`)
- ✅ Debug information (`-D`)

### Write Operations (Database Modification):
- ✅ Database description (`-d`)
- ✅ Compaction (`-C`) - removes deleted games, rewrites files
- ✅ Name compaction (`-N`) - removes unused names, rewrites files
- ✅ Sorting (`-S`, `-R`) - reorders games, rewrites index

### Key Technical Features:
- **Safe file operations** with backup and atomic swap pattern
- **Progress tracking** for long-running operations
- **Consistent error handling** across all operations
- **Memory-efficient** processing (reads in batches)
- **Preserves data integrity** with validation steps

### Entry Point for Implementers:
Use `scidt.cpp` as reference for:
1. How to open and read SCID database files
2. How to access game metadata through index entries
3. How to look up names from namebase
4. How to safely modify database files
5. How to handle various database operations efficiently

This document serves as the complete blueprint for all scidt operations and can be used to implement equivalent functionality in any programming language.
