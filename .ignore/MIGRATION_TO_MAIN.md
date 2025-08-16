# Migration Guide: Integrating Shakmaty-Based SCID Parser into Main Project

**Date:** August 14, 2025

## Overview
This guide describes the step-by-step process for migrating the shakmaty-based SCID parser from the experiments directory into the main codebase.

## Steps
1. **Review and Clean Up Lint Errors**
   - Address all remaining warnings and errors in the experiments code
2. **Move Bridge Layer and Core Modules**
   - Copy `bridge/`, `sg4.rs`, `si4.rs`, `sn4.rs`, `position.rs`, `notation.rs` to main source directory
3. **Integrate Tests**
   - Move all tests from `tests/` to main test suite
   - Ensure all tests pass in the main project context
4. **Update Main CLI**
   - Integrate CLI commands from `main.rs` into main application entry point
5. **Update Documentation**
   - Copy `ARCHITECTURE_IMPLEMENTED.md` and update main project docs
6. **Validate Against Reference Data**
   - Run full test suite and validate output against reference PGN files
7. **Deprecate Legacy Parsing**
   - Remove or archive legacy SCID parsing modules as needed

## Notes
- Migration should be performed incrementally and validated at each step
- See `PERFORMANCE_ANALYSIS.md` for optimization recommendations
- See `SCID_FORMAT_COMPLETION_STATUS.md` for feature completeness

---
**End of Migration Guide**
