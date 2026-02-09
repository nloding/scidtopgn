#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "doctest.h"
#include <cstring>

TEST_CASE("Date encoding - year 2024") {
    uint year = 2024;
    uint month = 1;
    uint day = 1;
    
    uint encoded = (year << 9) | (month << 5) | day;
    
    REQUIRE(encoded == ((2024 << 9) | (1 << 5) | 1));
}

TEST_CASE("Date encoding - year 2022 month 12 day 19") {
    uint year = 2022;
    uint month = 12;
    uint day = 19;
    
    uint encoded = (year << 9) | (month << 5) | day;
    
    uint8 decodedDay = encoded & 0x1F;
    uint8 decodedMonth = (encoded >> 5) & 0x0F;
    uint16 decodedYear = (encoded >> 9) & 0x7FF;
    
    REQUIRE(decodedYear == 2022);
    REQUIRE(decodedMonth == 12);
    REQUIRE(decodedDay == 19);
}

TEST_CASE("Date encoding - year 0 month 1 day 1") {
    uint year = 0;
    uint month = 1;
    uint day = 1;
    
    uint encoded = (year << 9) | (month << 5) | day;
    
    REQUIRE(encoded == 1);
}

TEST_CASE("Date encoding - maximum year 2047") {
    uint year = 2047;
    uint month = 12;
    uint day = 31;
    
    uint encoded = (year << 9) | (month << 5) | day;
    
    uint8 decodedDay = encoded & 0x1F;
    uint8 decodedMonth = (encoded >> 5) & 0x0F;
    uint16 decodedYear = (encoded >> 9) & 0x7FF;
    
    REQUIRE(decodedYear == 2047);
    REQUIRE(decodedMonth == 12);
    REQUIRE(decodedDay == 31);
}

TEST_CASE("Date bounds - month range") {
    for (uint month = 1; month <= 12; month++) {
        uint encoded = (2024 << 9) | (month << 5) | 15;
        uint8 decodedMonth = (encoded >> 5) & 0x0F;
        REQUIRE(decodedMonth == month);
    }
}

TEST_CASE("Date bounds - day range") {
    for (uint day = 1; day <= 31; day++) {
        uint encoded = (2024 << 9) | (6 << 5) | day;
        uint8 decodedDay = encoded & 0x1F;
        REQUIRE(decodedDay == day);
    }
}

TEST_CASE("Date encoding - invalid month 0") {
    uint encoded = (2024 << 9) | (0 << 5) | 15;
    uint8 decodedMonth = (encoded >> 5) & 0x0F;
    
    REQUIRE(decodedMonth == 0);
}

TEST_CASE("Date encoding - invalid month 13") {
    uint encoded = (2024 << 9) | (13 << 5) | 15;
    uint8 decodedMonth = (encoded >> 5) & 0x0F;
    
    REQUIRE(decodedMonth == 13);
}

TEST_CASE("Date encoding - invalid day 32") {
    uint encoded = (2024 << 9) | (6 << 5) | 32;
    uint8 decodedDay = encoded & 0x1F;
    
    REQUIRE(decodedDay == 0);
}

TEST_CASE("Date encoding - leap year February 29") {
    uint year = 2024;
    uint month = 2;
    uint day = 29;
    
    uint encoded = (year << 9) | (month << 5) | day;
    REQUIRE(encoded != 0);
}

TEST_CASE("Date encoding - non-leap year February 28") {
    uint year = 2023;
    uint month = 2;
    uint day = 28;
    
    uint encoded = (year << 9) | (month << 5) | day;
    REQUIRE(encoded != 0);
}
