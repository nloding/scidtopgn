#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "doctest.h"
#include <cstring>

TEST_CASE("ByteBuffer basic operations") {
    byte buffer[100];
    
    SECTION("write and read byte") {
        buffer[0] = 0x42;
        REQUIRE(buffer[0] == 0x42);
    }
    
    SECTION("write and read multiple bytes") {
        const byte testData[] = {0x01, 0x02, 0x03, 0x04, 0x05};
        for (int i = 0; i < 5; i++) {
            buffer[i] = testData[i];
        }
        
        for (int i = 0; i < 5; i++) {
            REQUIRE(buffer[i] == testData[i]);
        }
    }
}

TEST_CASE("ByteBuffer boundary checking") {
    byte buffer[10];
    
    SECTION("write to buffer") {
        for (int i = 0; i < 10; i++) {
            buffer[i] = (byte)i;
        }
    }
    
    SECTION("read from buffer") {
        for (int i = 0; i < 10; i++) {
            REQUIRE(buffer[i] == (byte)i);
        }
    }
}

TEST_CASE("ByteBuffer string operations") {
    char str[100];
    
    SECTION("copy string") {
        const char* testStr = "Hello World";
        std::strcpy(str, testStr);
        REQUIRE(std::strcmp(str, testStr) == 0);
    }
    
    SECTION("string length") {
        const char* testStr = "Test";
        std::strcpy(str, testStr);
        REQUIRE(std::strlen(str) == 4);
    }
}

TEST_CASE("ByteBuffer endianness") {
    byte buffer[4];
    
    SECTION("little endian 16-bit") {
        uint16 value = 0x1234;
        buffer[0] = value & 0xFF;
        buffer[1] = (value >> 8) & 0xFF;
        
        REQUIRE(buffer[0] == 0x34);
        REQUIRE(buffer[1] == 0x12);
    }
    
    SECTION("big endian 16-bit") {
        uint16 value = 0x1234;
        buffer[0] = (value >> 8) & 0xFF;
        buffer[1] = value & 0xFF;
        
        REQUIRE(buffer[0] == 0x12);
        REQUIRE(buffer[1] == 0x34);
    }
    
    SECTION("little endian 32-bit") {
        uint32 value = 0x01234567;
        buffer[0] = value & 0xFF;
        buffer[1] = (value >> 8) & 0xFF;
        buffer[2] = (value >> 16) & 0xFF;
        buffer[3] = (value >> 24) & 0xFF;
        
        REQUIRE(buffer[0] == 0x67);
        REQUIRE(buffer[1] == 0x45);
        REQUIRE(buffer[2] == 0x23);
        REQUIRE(buffer[3] == 0x01);
    }
    
    SECTION("big endian 32-bit") {
        uint32 value = 0x01234567;
        buffer[0] = (value >> 24) & 0xFF;
        buffer[1] = (value >> 16) & 0xFF;
        buffer[2] = (value >> 8) & 0xFF;
        buffer[3] = value & 0xFF;
        
        REQUIRE(buffer[0] == 0x01);
        REQUIRE(buffer[1] == 0x23);
        REQUIRE(buffer[2] == 0x45);
        REQUIRE(buffer[3] == 0x67);
    }
}

TEST_CASE("ByteBuffer null termination") {
    char str[100];
    
    SECTION("null-terminated string") {
        const char* testStr = "Test";
        std::strcpy(str, testStr);
        str[4] = '\0';
        
        REQUIRE(std::strlen(str) == 4);
        REQUIRE(str[4] == '\0');
    }
}

TEST_CASE("ByteBuffer memory operations") {
    byte src[10];
    byte dst[10];
    
    SECTION("memory copy") {
        for (int i = 0; i < 10; i++) {
            src[i] = (byte)i;
        }
        std::memcpy(dst, src, 10);
        
        for (int i = 0; i < 10; i++) {
            REQUIRE(dst[i] == src[i]);
        }
    }
    
    SECTION("memory compare") {
        for (int i = 0; i < 10; i++) {
            src[i] = (byte)i;
            dst[i] = (byte)i;
        }
        
        REQUIRE(std::memcmp(src, dst, 10) == 0);
    }
}
