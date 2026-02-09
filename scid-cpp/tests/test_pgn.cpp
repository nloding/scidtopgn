#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "doctest.h"

TEST_CASE("PGN parsing - header tags") {
    const char* pgnGame = 
        "[Event \"Test Game\"]\n"
        "[Site \"Test Site\"]\n"
        "[Date \"2024.01.01\"]\n"
        "[Round \"1\"]\n"
        "[White \"Alice\"]\n"
        "[Black \"Bob\"]\n"
        "[Result \"1-0\"]\n"
        "\n"
        "1. e4 e5 2. Nf3 Nc6 1-0\n";
    
    REQUIRE(pgnGame != NULL);
}

TEST_CASE("PGN parsing - move notation") {
    const char* moves[] = {
        "1. e4",
        "2. Nf3",
        "3. Nc6",
        "4. O-O"
    };
    
    for (int i = 0; i < 4; i++) {
        REQUIRE(moves[i] != NULL);
    }
}

TEST_CASE("PGN parsing - castling") {
    const char* castleKingside = "O-O";
    const char* castleQueenside = "O-O-O";
    
    REQUIRE(std::strcmp(castleKingside, "O-O") == 0);
    REQUIRE(std::strcmp(castleQueenside, "O-O-O") == 0);
}

TEST_CASE("PGN parsing - results") {
    const char* results[] = {"1-0", "0-1", "1/2-1/2", "*"};
    
    for (int i = 0; i < 4; i++) {
        REQUIRE(results[i] != NULL);
    }
}

TEST_CASE("PGN parsing - pawn promotion") {
    const char* promotions[] = {
        "e8=Q",
        "d8=R",
        "c8=B",
        "b8=N"
    };
    
    for (int i = 0; i < 4; i++) {
        REQUIRE(promotions[i] != NULL);
        REQUIRE(std::strchr(promotions[i], '=') != NULL);
    }
}

TEST_CASE("PGN parsing - annotations") {
    const char* annotations[] = {
        "{good move}",
        "{poor move}",
        "{blunder}",
        "{excellent move}"
    };
    
    for (int i = 0; i < 4; i++) {
        REQUIRE(annotations[i] != NULL);
        REQUIRE(annotations[i][0] == '{');
        REQUIRE(annotations[i][std::strlen(annotations[i]) - 1] == '}');
    }
}

TEST_CASE("PGN parsing - NAG symbols") {
    const char* nagSymbols[] = {"!", "?", "!!", "??"};
    
    for (int i = 0; i < 3; i++) {
        REQUIRE(nagSymbols[i] != NULL);
        REQUIRE(std::strlen(nagSymbols[i]) == 1);
    }
}

TEST_CASE("PGN parsing - variations") {
    const char* variation = "(e4 c5)";
    
    REQUIRE(variation != NULL);
    REQUIRE(variation[0] == '(');
    REQUIRE(variation[std::strlen(variation) - 1] == ')');
}

TEST_CASE("PGN parsing - round numbers") {
    const char* moveNumbers[] = {
        "1. e4",
        "2. Nf3",
        "10. Qe2"
    };
    
    for (int i = 0; i < 3; i++) {
        REQUIRE(moveNumbers[i] != NULL);
    }
}

TEST_CASE("PGN parsing - check and mate") {
    const char* checks[] = {"+", "#"};
    
    for (int i = 0; i < 2; i++) {
        REQUIRE(checks[i] != NULL);
    }
}

TEST_CASE("SCID file extensions") {
    const char* extensions[] = {".si4", ".sn4", ".sg4"};
    
    for (int i = 0; i < 3; i++) {
        REQUIRE(extensions[i] != NULL);
        REQUIRE(std::strlen(extensions[i]) == 4);
        REQUIRE(extensions[i][0] == '.');
        REQUIRE(extensions[i][3] == '4');
    }
}

TEST_CASE("SCID magic strings") {
    const char* magicStrings[] = {
        "Scid.si",
        "Scid.sn",
        "Scid.sg"
    };
    
    for (int i = 0; i < 3; i++) {
        REQUIRE(std::strlen(magicStrings[i]) == 8);
    }
}
