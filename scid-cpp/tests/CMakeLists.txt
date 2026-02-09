cmake_minimum_required(VERSION 3.15)
project(scidtests LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add test sources
enable_testing()

# Test executables
add_executable(test_bytebuf test_bytebuf.cpp)
target_include_directories(test_bytebuf PRIVATE ${CMAKE_SOURCE_DIR})
target_link_libraries(test_bytebuf PRIVATE PRIVATE doctest::doctest)
add_test(NAME test_bytebuf COMMAND test_bytebuf)

add_executable(test_date test_date.cpp)
target_include_directories(test_date PRIVATE ${CMAKE_SOURCE_DIR})
target_link_libraries(test_date PRIVATE doctest::doctest)
add_test(NAME test_date COMMAND test_date)

add_executable(test_namebase test_namebase.cpp)
target_include_directories(test_namebase PRIVATE ${CMAKE_SOURCE_DIR})
target_link_libraries(test_namebase PRIVATE doctest::doctest)
add_test(NAME test_namebase COMMAND test_namebase)

add_executable(test_pgn test_pgn.cpp)
target_include_directories(test_pgn PRIVATE ${CMAKE_SOURCE_DIR})
target_link_libraries(test_pgn PRIVATE doctest::doctest)
add_test(NAME test_pgn COMMAND test_pgn)
