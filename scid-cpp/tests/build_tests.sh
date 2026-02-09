#!/bin/bash

# Simple build script for tests (when CMake is not available)

echo "Building test executables..."

# Compile test_bytebuf
echo "  Compiling test_bytebuf.cpp..."
g++ -std=c++17 -I.. -I../scid test_bytebuf.cpp -o test_bytebuf

# Compile test_date
echo "  Compiling test_date.cpp..."
g++ -std=c++17 -I.. -I../scid test_date.cpp -o test_date

# Compile test_namebase
echo "  Compiling test_namebase.cpp..."
g++ -std=c++17 -I.. -I../scid test_namebase.cpp -o test_namebase

# Compile test_pgn
echo "  Compiling test_pgn.cpp..."
g++ -std=c++17 -I.. -I../scid test_pgn.cpp -o test_pgn

if [ $? -eq 0 ]; then
    echo -e "\033[0;32mAll tests compiled successfully!\033[0m"
    echo ""
    echo "Running tests..."
    echo ""
    
    # Run all tests
    ./test_bytebuf
    bytebuf_status=$?
    ./test_date
    date_status=$?
    ./test_namebase
    namebase_status=$?
    ./test_pgn
    pgn_status=$?
    
    # Check results
    total_status=$((bytebuf_status + date_status + namebase_status + pgn_status))
    
    if [ $total_status -eq 0 ]; then
        echo -e "\033[0;32mAll tests passed!\033[0m"
    else
        echo -e "\033[0;31mSome tests failed!\033[0m"
    fi
else
    echo -e "\033[0;31mCompilation failed!\033[0m"
fi
