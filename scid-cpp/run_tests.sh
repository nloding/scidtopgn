#!/bin/bash

# Test runner script for scid-cpp

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building scid-cpp...${NC}"

# Create build directory
mkdir -p build
cd build

# Configure
echo -e "${GREEN}Configuring with CMake...${NC}"
cmake ..

# Build
echo -e "${GREEN}Building...${NC}"
make -j$(nproc)

# Check build status
if [ $? -eq 0 ]; then
    echo -e "${GREEN}Build successful!${NC}"
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

# Run tests if they exist
if [ -f "test_bytebuf" ]; then
    echo -e "${GREEN}Running tests...${NC}"
    ctest --verbose
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
    else
        echo -e "${RED}Some tests failed!${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}No test executables found. Run 'make' in build/ directory first.${NC}"
fi

echo -e "${GREEN}Done!${NC}"
