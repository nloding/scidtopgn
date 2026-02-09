# Unit Tests for scid-cpp

This directory contains comprehensive unit tests for the SCID to PGN converter.

## Test Files

| Test File | Description |
|-----------|-------------|
| `test_bytebuf.cpp` | ByteBuffer operations (PutByte, GetByte, buffer boundaries, endianness) |
| `test_date.cpp` | Date encoding/decoding (absolute and relative date formats) |
| `test_namebase.cpp` | NameBase operations (add names, lookups, compression, special characters) |
| `test_pgn.cpp` | PGN parsing (tags, moves, annotations, variations) |

## Building Tests

From the `scid-cpp/tests` directory:

```bash
# Create build directory
cd ..
mkdir -p build
cd build

# Configure with CMake
cmake ..

# Build tests
make

# Run all tests
ctest --verbose

# Run specific test
./test_bytebuf
```

## Test Categories

### 1. ByteBuffer Tests (`test_bytebuf.cpp`)
- Basic read/write operations
- Buffer boundary checking
- Multi-byte operations (Put2Bytes, Put3Bytes, Put4Bytes)
- String operations (PutTerminatedString, PutFixedString)
- Buffer navigation (BackToStart, Skip)
- Memory operations (CopyTo, CopyFrom)

### 2. Date Tests (`test_date.cpp`)
- Absolute date encoding (year << 9 | month << 5 | day)
- Date bounds checking (valid ranges)
- Leap year handling
- Month validation (1-12)
- Day validation (1-31)
- Year range (0-2047)

### 3. NameBase Tests (`test_namebase.cpp`)
- Name adding (players, events, sites, rounds)
- Name lookup by ID
- Duplicate name detection
- Special characters (hyphens, apostrophes)
- Case sensitivity
- Unicode handling
- Prefix compression testing

### 4. PGN Tests (`test_pgn.cpp`)
- PGN header tags parsing
- Move notation (SAN)
- Castling (O-O, O-O-O)
- Pawn promotions (e8=Q, etc.)
- Annotations ({comment}, NAG symbols)
- Variations (move alternatives)
- Check and mate symbols
- Round numbers
- SCID file format validation

## Running Tests Individually

```bash
# Run all tests
ctest --verbose

# Run specific test file
./test_bytebuf
./test_date
./test_namebase
./test_pgn

# Run with specific test case filter
ctest -R "ByteBuffer"
ctest -R "Date"
```

## Expected Results

All tests should pass with no failures. The test suite covers:

- ✅ Critical buffer operations
- ✅ Date encoding accuracy
- ✅ Name management
- ✅ PGN format validation
- ✅ Edge cases and boundary conditions

## Adding New Tests

To add a new test:

1. Create a new test file (e.g., `test_game.cpp`)
2. Add test cases using `TEST_CASE("test name")` syntax
3. Update `CMakeLists.txt` to include the new test
4. Rebuild and run `ctest`

Example:
```cpp
TEST_CASE("My new test") {
    REQUIRE(1 + 1 == 2);
}
```

## CI/CD Integration

For continuous integration:

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build and test
        run: |
          cd scid-cpp
          mkdir -p build && cd build
          cmake ..
          make
          ctest --verbose
```

## Dependencies

- CMake 3.15 or later
- C++17 compatible compiler
- doctest (included as single header)

## Test Coverage

To measure test coverage:

```bash
# Install gcov and lcov
sudo apt-get install gcov lcov

# Build with coverage flags
cmake -DCMAKE_CXX_FLAGS="--coverage" ..
make

# Run tests
ctest

# Generate coverage report
lcov --capture --directory . --output-file coverage.info --base-directory . .
lcov --output coverage.info --html
```

Open `coverage/index.html` in a browser to view the report.

## License

Tests are part of the scidtopgn project and follow the same license terms.
