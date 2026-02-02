# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of scidtopng library and CLI tool
- Complete SCID database parsing (.si4, .sn4, .sg4 files)
- Support for game index parsing with metadata
- Front-coding decompression for name file
- Move decoding with shakmaty integration
- PGN generation with SAN notation
- Streaming API for memory-efficient handling of large databases
- Comprehensive error handling for corrupted data
- CLI tool with filtering and conversion options
- API documentation with examples
- Performance benchmarks

### Known Issues
- In progress: Add more test coverage for edge cases
- In progress: Optimize performance for very large databases

---

## [0.1.0] - 2026-02-01

### Added
- Complete SCID database parser for .si4, .sn4, .sg4 files
- Memory-efficient streaming API for large databases
- Support for reading game metadata (players, date, result, ELO)
- Front-coding decompression for compressed name data
- Move decoding using shakmaty library
- PGN generation with standard seven tag roster
- Error handling for I/O and parse errors
- CLI tool for command-line conversion
- Library API for programmatic access
- Comprehensive documentation
