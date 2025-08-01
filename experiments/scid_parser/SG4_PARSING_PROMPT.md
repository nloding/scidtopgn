# SG4 PARSING PROMPT

This document provides a precise methodology for implementing SCID .sg4 game file parsing. This approach was developed after an initial failed attempt where assumptions were made without proper validation, resulting in incorrect implementation that had to be rolled back. The successful methodology requires step-by-step validation against SCID source code.

## CRITICAL REQUIREMENTS

### ❌ DO NOT DO THESE THINGS:
- **DO NOT** assume byte order (little vs big endian) without checking SCID source code
- **DO NOT** implement full parsing logic in the first attempt
- **DO NOT** create new CLI arguments or modify existing command structure
- **DO NOT** try to parse individual bytes until display structures are ready
- **DO NOT** make assumptions about field sizes, formats, or structures
- **DO NOT** skip validation against scidvspc source code

### ✅ MANDATORY APPROACH:
- **ALWAYS** use big-endian byte order (consistent with .si4 and .sn4 implementations)
- **ALWAYS** validate every step against scidvspc source code before and after implementation
- **ALWAYS** work incrementally - display first, then parse field by field
- **ALWAYS** follow the baby steps approach that proved successful
- **ALWAYS** add output to existing parse command (no new CLI arguments)

## STEP-BY-STEP METHODOLOGY

### PHASE 1: Research and Structure Analysis
1. **Analyze SCID source code FIRST**:
   - Read `/Users/nloding/code/scidtopgn/scidvspc/src/gfile.cpp` thoroughly
   - Read `/Users/nloding/code/scidtopgn/scidvspc/src/gfile.h` for structures
   - Identify exact field names, sizes, and byte order from source
   - Document findings before writing any code

2. **Create structure documentation table**:
   - Create `display_sg4_structure()` function showing byte layout
   - Use same format as successful si4/sn4 structure tables
   - Show offset, size, format, and field description
   - Include notes about endianness and special encodings

### PHASE 2: Basic Framework Setup
3. **Create basic sg4.rs module structure**:
   - Add basic header structure based on SCID source analysis
   - Create display functions for structure tables only
   - Add sg4 parsing to existing parse command in main.rs
   - Show structure tables but parse NO individual bytes yet

4. **Validate structure display**:
   - Test that structure tables display correctly
   - Verify field names match SCID source code exactly
   - Confirm byte offsets and sizes are accurate
   - User validates approach before proceeding

### PHASE 3: Incremental Field Parsing
5. **Parse ONE field at a time**:
   - Start with the simplest field (usually a header field)
   - Read ONLY that field using appropriate big-endian function
   - Display field value in table format with placeholder for others
   - Validate implementation against SCID source code

6. **Validate each field implementation**:
   - Compare parsing logic with exact SCID source code
   - Test with known data to verify correct values
   - User reviews and approves before moving to next field
   - Keep implementation clean and ready for other fields

7. **Repeat for each field systematically**:
   - Move through fields one by one in logical order
   - Always validate against SCID source before implementation
   - Always test with real data after implementation
   - Never implement multiple fields without validation

### PHASE 4: Integration and Verification
8. **Complete parsing integration**:
   - Ensure all fields work together correctly
   - Test with multiple games from test data
   - Validate output against expected PGN game data
   - Cross-reference with existing .si4/.sn4 parsed data

9. **Final validation**:
   - Double-check implementation against SCID source code
   - Verify all field values make sense for chess games
   - Confirm big-endian byte order throughout
   - Test edge cases and error handling

## SPECIFIC IMPLEMENTATION REQUIREMENTS

### File Structure:
- Work in `/Users/nloding/code/scidtopgn/experiments/scid_parser/src/sg4.rs`
- Add sg4 parsing to existing parse command in main.rs
- Use same table display format as si4.rs and sn4.rs

### Code Style:
- Follow exact patterns from successful sn4.rs implementation
- Use big-endian read functions: `read_u8()`, `read_u16_be()`, `read_u24_be()`, `read_u32_be()`
- Include comprehensive comments referencing SCID source code
- Clean, readable code ready for integration

### Data Validation:
- Use test data: `/Users/nloding/code/scidtopgn/test/data/five.sg4`
- Cross-validate with `/Users/nloding/code/scidtopgn/test/data/five.pgn`
- Ensure parsed game data matches expected chess moves and metadata

## SCID SOURCE CODE REFERENCES

### Key Files to Analyze:
- **gfile.cpp**: Main game file parsing logic
- **gfile.h**: Game file structures and constants
- **game.cpp**: Game representation and move encoding
- **position.cpp**: Chess position and move validation

### Focus Areas:
- Game header structure and fields
- Move encoding format (typically 2-3 bytes per move)
- Variation and comment storage
- NAG (Numeric Annotation Glyph) handling
- Custom starting positions
- End-of-game markers

## SUCCESS CRITERIA

The implementation is successful when:
1. ✅ All fields parse correctly with meaningful values
2. ✅ Output matches SCID source code logic exactly
3. ✅ Big-endian byte order used throughout
4. ✅ Chess moves can be reconstructed from parsed data
5. ✅ Values cross-validate with .si4 index data
6. ✅ Code is clean and ready for integration

## FAILURE INDICATORS

Stop and restart if:
- ❌ Making assumptions about field formats
- ❌ Getting unrealistic values (like impossible dates/moves)
- ❌ Not validating against SCID source code
- ❌ Trying to implement too many fields at once
- ❌ Using little-endian when big-endian is required

## EXAMPLE WORKFLOW

```
1. Analyze gfile.cpp GameFile::ReadGame() function
2. Create display_sg4_structure() showing all fields  
3. Add basic sg4 parsing to main.rs parse command
4. Parse ONLY game length field first
5. Validate game length against SCID source
6. Parse ONLY move count field second  
7. Validate move count parsing
8. Continue field by field...
9. Final integration and testing
```

## CONTEXT

This prompt was created after a failed first attempt where:
- Big-endian byte order was not used consistently
- SCID source code was not consulted before implementation
- Too many fields were implemented simultaneously
- Validation was skipped, leading to incorrect parsing
- The entire implementation had to be discarded

The successful methodology (used for .sn4 parsing) resulted in:
- 100% accurate parsing of all fields
- Perfect validation against test data
- Clean, maintainable code
- Front-coded string reconstruction working perfectly

**Follow this methodology exactly to achieve the same success with .sg4 parsing.**