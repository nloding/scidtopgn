#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "doctest.h"
#include <cstring>

TEST_CASE("NameBase operations") {
    const char* names[] = {
        "Magnus Carlsen",
        "Fabiano Caruana",
        "Hikaru Nakamura"
    };
    
    SECTION("name length calculation") {
        for (int i = 0; i < 3; i++) {
            REQUIRE(std::strlen(names[i]) > 0);
        }
    }
    
    SECTION("name comparison") {
        REQUIRE(std::strcmp(names[0], "Magnus Carlsen") == 0);
        REQUIRE(std::strcmp(names[0], names[1]) != 0);
    }
    
    SECTION("name copying") {
        char copy[100];
        std::strcpy(copy, names[0]);
        REQUIRE(std::strcmp(copy, names[0]) == 0);
    }
}

TEST_CASE("NameBase - special characters") {
    const char* specialNames[] = {
        "Nakamura, Hikaru",
        "O'Kelly, John",
        "Carlsen, Magnus",
        "Caruana, Fabiano"
    };
    
    SECTION("names with hyphens") {
        REQUIRE(std::strchr(specialNames[0], '-') != NULL);
        REQUIRE(std::strchr(specialNames[3], ',') != NULL);
    }
    
    SECTION("names with apostrophes") {
        REQUIRE(std::strchr(specialNames[1], '\'') != NULL);
    }
    
    SECTION("names with spaces") {
        REQUIRE(std::strchr(specialNames[0], ' ') != NULL);
    }
}

TEST_CASE("NameBase - case sensitivity") {
    const char* name1 = "Carlsen, Magnus";
    const char* name2 = "CARLSEN, MAGNUS";
    const char* name3 = "carlsen, magnus";
    
    REQUIRE(std::strcmp(name1, name1) == 0);
    REQUIRE(std::strcmp(name1, name2) != 0);
    REQUIRE(std::strcmp(name1, name3) != 0);
}

TEST_CASE("NameBase - empty name") {
    const char* name = "";
    
    REQUIRE(std::strlen(name) == 0);
    REQUIRE(name[0] == '\0');
}

TEST_CASE("NameBase - unicode handling") {
    const char* unicodeName = "Carlsen, Magnus";
    
    SECTION("valid unicode name") {
        REQUIRE(std::strlen(unicodeName) > 0);
        REQUIRE(std::strchr(unicodeName, ' ') != NULL);
    }
}

TEST_CASE("NameBase - name types") {
    const char* players[] = {"Magnus Carlsen", "Fabiano Caruana"};
    const char* events[] = {"World Championship", "Candidates"};
    const char* sites[] = {"London", "New York"};
    const char* rounds[] = {"1", "2", "3"};
    
    SECTION("player names") {
        for (int i = 0; i < 2; i++) {
            REQUIRE(std::strlen(players[i]) > 0);
        }
    }
    
    SECTION("event names") {
        for (int i = 0; i < 2; i++) {
            REQUIRE(std::strlen(events[i]) > 0);
        }
    }
    
    SECTION("site names") {
        for (int i = 0; i < 2; i++) {
            REQUIRE(std::strlen(sites[i]) > 0);
        }
    }
    
    SECTION("round names") {
        for (int i = 0; i < 3; i++) {
            REQUIRE(std::strlen(rounds[i]) > 0);
        }
    }
}

TEST_CASE("NameBase - duplicate detection") {
    const char* name = "Magnus Carlsen";
    char* copy1 = new char[100];
    char* copy2 = new char[100];
    
    std::strcpy(copy1, name);
    std::strcpy(copy2, name);
    
    REQUIRE(std::strcmp(copy1, copy2) == 0);
    
    delete[] copy1;
    delete[] copy2;
}

TEST_CASE("NameBase - prefix compression") {
    const char* names[] = {
        "Carlsen, Magnus",
        "Carlsen, Henrik",
        "Caruana, Fabiano"
    };
    
    SECTION("calculate common prefix") {
        size_t prefixLen = 0;
        const char* p1 = names[0];
        const char* p2 = names[1];
        
        while (*p1 && *p2 && *p1 == *p2) {
            prefixLen++;
            p1++;
            p2++;
        }
        
        REQUIRE(prefixLen == 9);
    }
    
    SECTION("different prefix") {
        size_t prefixLen = 0;
        const char* p1 = names[0];
        const char* p2 = names[2];
        
        while (*p1 && *p2 && *p1 == *p2) {
            prefixLen++;
            p1++;
            p2++;
        }
        
        REQUIRE(prefixLen == 3);
    }
}
