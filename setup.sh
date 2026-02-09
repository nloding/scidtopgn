#!/bin/bash

# Setup script for scid-cpp project
# Copies required SCID source files to self-contained directory

set -e  # Exit on error

# Source directory (original SCID code)
SCID_SOURCE_DIR="./scidvspc/scidvspc-main/src"

# Target directory (self-contained)
TARGET_DIR="./scid-cpp"
SCID_DIR="${TARGET_DIR}/scid"

echo "Setting up scid-cpp project..."

# Create directories
mkdir -p "${TARGET_DIR}/src"
mkdir -p "${SCID_DIR}"

# Files to copy (read-only from original source)
SCID_FILES=(
    "common.h"
    "error.h"
    "myassert.h"
    "myassert.cpp"
    "mfile.cpp"
    "mfile.h"
    "bytebuf.cpp"
    "bytebuf.h"
    "index.cpp"
    "index.h"
    "namebase.cpp"
    "namebase.h"
    "game.cpp"
    "game.h"
    "gfile.cpp"
    "gfile.h"
    "position.cpp"
    "position.h"
    "date.cpp"
    "date.h"
    "misc.cpp"
    "misc.h"
    "stralloc.cpp"
    "stralloc.h"
    "textbuf.cpp"
    "textbuf.h"
    "movelist.cpp"
    "movelist.h"
    "sqmove.h"
    "sqlist.h"
    "sqset.h"
    "attacks.h"
    "dstring.cpp"
    "dstring.h"
    "hash.h"
    "matsig.cpp"
    "matsig.h"
    "nagtext.h"
    "naglatex.h"
    "pgnparse.h"
    "pgnparse.cpp"
    "stored.cpp"
    "stored.h"
    "strtree.h"
    "tokens.h"
    "recog.h"
    "recog.cpp"
    "optable.h"
    "optable.cpp"
    "pbook.h"
    "pbook.cpp"
    "charsetdetector.h"
    "charsetdetector.cpp"
    "charsetconverter.h"
    "charsetconverter.cpp"
    "engine.h"
    "timer.h"
)

# Copy SCID files
echo "Copying SCID source files..."
for file in "${SCID_FILES[@]}"; do
    if [ -f "${SCID_SOURCE_DIR}/${file}" ]; then
        cp "${SCID_SOURCE_DIR}/${file}" "${SCID_DIR}/${file}"
        echo "  ✓ ${file}"
    else
        echo "  ✗ NOT FOUND: ${file}"
        exit 1
    fi
done

echo ""
echo "✓ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Copy main.cpp to ${TARGET_DIR}/src/"
echo "  2. Copy CMakeLists.txt to ${TARGET_DIR}/"
echo "  3. Copy README.md to ${TARGET_DIR}/"
echo "  4. cd ${TARGET_DIR}/build"
echo "  5. cmake .."
echo "  6. make"
