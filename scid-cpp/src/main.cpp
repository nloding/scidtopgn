#include "index.h"
#include "namebase.h"
#include "gfile.h"
#include "pgnparse.h"
#include "game.h"
#include "textbuf.h"
#include "error.h"
#include "mfile.h"

#include <iostream>
#include <cstring>

int main(int argc, char* argv[]) {
    // Check arguments
    if (argc < 2 || argc > 3) {
        std::cerr << "Usage: scidtopgn <database> [pgn_file]" << std::endl;
        std::cerr << "  For reading SCID to PGN: scidtopgn <database>" << std::endl;
        std::cerr << "  For writing PGN to SCID: scidtopgn <database> <pgn_file>" << std::endl;
        return 1;
    }

    const char* database_path = argv[1];
    const char* pgn_file = (argc == 3) ? argv[2] : NULL;

    errorT err;

    // Mode 1: SCID -> PGN (existing functionality)
    if (pgn_file == NULL) {
        // Initialize SCID objects
        Index* idx = new Index();
        NameBase* nb = new NameBase();
        GFile* gf = new GFile();

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
        err = gf->Open(database_path, FMODE_ReadOnly);
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
            err = game.LoadStandardTags(entry, nb);
            if (err != OK) {
                std::cerr << "Warning: error loading tags for game " << (gnum + 1) << std::endl;
                continue;
            }

            // Set PGN format and style
            game.SetPgnFormat(PGN_FORMAT_Plain);
            game.SetPgnStyle(PGN_STYLE_TAGS, true);
            game.SetPgnStyle(PGN_STYLE_COMMENTS, true);
            game.SetPgnStyle(PGN_STYLE_VARS, true);
            game.SetPgnStyle(PGN_STYLE_SYMBOLS, true);

            // Write game to PGN format into TextBuffer
            TextBuffer tb;
            tb.SetBufferSize(10000);
            tb.SetWrapColumn(10000);
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

    // Mode 2: PGN -> SCID (new functionality)
    else {
        // Open PGN file for reading
        MFile* pgnInput = new MFile();
        err = pgnInput->Open(pgn_file, FMODE_ReadOnly);
        if (err != OK) {
            std::cerr << "Error: cannot open PGN file: " << pgn_file << std::endl;
            delete pgnInput;
            return 1;
        }

        // Initialize SCID objects for writing
        Index* idx = new Index();
        NameBase* nb = new NameBase();
        GFile* gf = new GFile();

        // Set filename (adds .si4, .sn4, .sg4 automatically)
        idx->SetFileName(database_path);
        nb->SetFileName(database_path);

        // Create new index file
        err = idx->CreateIndexFile(FMODE_Both);
        if (err != OK) {
            std::cerr << "Error: cannot create index file (.si4)" << std::endl;
            delete pgnInput;
            delete idx;
            delete nb;
            delete gf;
            return 1;
        }

        // Create new game file
        err = gf->Create(database_path, FMODE_Both);
        if (err != OK) {
            std::cerr << "Error: cannot create game file (.sg4)" << std::endl;
            delete pgnInput;
            idx->CloseIndexFile();
            delete idx;
            delete nb;
            delete gf;
            return 1;
        }

        // Initialize NameBase first, then set filename
        nb->Init();
        nb->SetFileName(database_path);

        // Parse PGN games and add to database
        PgnParser parser(pgnInput);
        gameNumberT gamesAdded = 0;
    int iteration = 0;

        while (true) {
            // Clear game object
            Game game;
            game.Clear();
            game.Init();

            // Parse next game from PGN
            err = parser.ParseGame(&game);

            const char* ws = game.GetWhiteStr();
            std::cerr << "After ParseGame - WhiteStr: [" << (ws ? ws : "NULL") << "]" << std::endl;
            std::cerr << "                    BlackStr: [" << (game.GetBlackStr() ? game.GetBlackStr() : "NULL") << "]" << std::endl;
            std::cerr << "                    EventStr: [" << (game.GetEventStr() ? game.GetEventStr() : "NULL") << "]" << std::endl;
            std::cerr << "                    SiteStr: [" << (game.GetSiteStr() ? game.GetSiteStr() : "NULL") << "]" << std::endl;
            std::cerr << "                    RoundStr: [" << (game.GetRoundStr() ? game.GetRoundStr() : "NULL") << "]" << std::endl;


            if (err == ERROR_NotFound) {
                // No more games
                break;
            }

            if (err != OK) {
                std::cerr << "Warning: error parsing game " << (gamesAdded + 1) << std::endl;
                continue;
            }

            // Add player/event/site names to NameBase
            idNumberT whiteId, blackId, eventId, siteId, roundId;
            nb->AddName(NAME_PLAYER, game.GetWhiteStr(), &whiteId);
            const char* wstr = game.GetWhiteStr();
            nb->AddName(NAME_PLAYER, game.GetBlackStr(), &blackId);
            const char* bstr = game.GetBlackStr();
            nb->AddName(NAME_EVENT, game.GetEventStr(), &eventId);
            nb->AddName(NAME_SITE, game.GetSiteStr(), &siteId);
            nb->AddName(NAME_ROUND, game.GetRoundStr(), &roundId);

            // Prepare IndexEntry
            IndexEntry entry;
            entry.Init();
            entry.SetWhite(whiteId);
            entry.SetBlack(blackId);
            entry.SetEvent(eventId);
            entry.SetSite(siteId);
            entry.SetRound(roundId);
            entry.SetResult(game.GetResult());
            entry.SetDate(game.GetDate());
            entry.SetWhiteElo(game.GetWhiteElo());
            entry.SetBlackElo(game.GetBlackElo());
            entry.SetEcoCode(game.GetEco());
            entry.SetNumHalfMoves(game.GetNumHalfMoves());

            // Add index entry
            gameNumberT gameNum;
            err = idx->AddGame(&gameNum, &entry, false);  // false = don't init (already init'd)
            if (err != OK) {
                std::cerr << "Error: cannot add game to index" << std::endl;
                break;
            }

            // Encode game to ByteBuffer
            ByteBuffer bb;
            // Encode() from scidvspc supports writing PGN→SCID games correctly
            game.Encode(&bb, &entry);

            // Add game to .sg4 file
            uint offset;
            err = gf->AddGame(&bb, &offset);
            if (err != OK) {
                std::cerr << "Error: cannot add game to game file" << std::endl;
                break;
            }

            // Update index entry with offset
            entry.SetOffset(offset);
            entry.SetLength(bb.GetByteCount());

            // Write index entry
            err = idx->WriteEntries(&entry, gameNum, 1);
            if (err != OK) {
                std::cerr << "Error: cannot write index entry" << std::endl;
                break;
            }

            gamesAdded++;
            
            iteration++;
            
            // Debug break removed
        }

        // Write name file
        err = nb->WriteNameFile();
        char sn4path[300];
        strcpy(sn4path, database_path);
        strcat(sn4path, ".sn4");
        FILE* f = fopen(sn4path, "rb");
        if (f) {
            fseek(f, 0, SEEK_END);
            long size = ftell(f);
            fclose(f);
            std::cerr << "  NumPlayers: " << nb->GetNumNames(0) << std::endl;
            std::cerr << "  NumEvents: " << nb->GetNumNames(1) << std::endl;
            std::cerr << "  NumSites: " << nb->GetNumNames(2) << std::endl;
            std::cerr << "  NumRounds: " << nb->GetNumNames(3) << std::endl;
        } else {
        }
        if (err != OK) {
            std::cerr << "Warning: error writing name file (.sn4)" << std::endl;
        }

        // Close files
        pgnInput->Close();
        idx->CloseIndexFile();
        gf->Close();

        // Cleanup
        delete pgnInput;
        delete idx;
        delete nb;
        delete gf;

        std::cerr << "Successfully added " << gamesAdded << " games to database." << std::endl;
        return 0;
    }
}
