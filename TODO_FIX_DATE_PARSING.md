# TODO: Fix Date Parsing Implementation

**Priority**: 🚨 **CRITICAL**  
**Status**: **RESEARCH COMPLETE** ✅ - Ready for implementation  
**Estimated Effort**: 1-2 days (research phase complete)  

## Overview

This TODO addresses the critical hardcoded date parsing issue documented in `DATE_PARSING_ISSUE.md`. **RESEARCH PHASE IS COMPLETE** - we now have a full understanding of the SCID date format and a working reference implementation.

**Key Finding**: The date field contains **BOTH** game date AND event date in a single 32-bit value using bit packing. The current implementation only extracts the game date and ignores the event date.

---

## ✅ Research Phase - COMPLETED

### ✅ 1.1 SCID Format Analysis - DONE
- **Analyzed SCID source code**: `scidvspc/src/index.cpp`, `index.h`, `date.h`
- **Documented exact 47-byte game index structure**: See `SCID_DATABASE_FORMAT.md`
- **Confirmed date field location**: Always at offset 25-28 (4 bytes)
- **Discovered dual date encoding**: 32-bit field contains both dates

### ✅ 1.2 Date Encoding Understanding - DONE  
- **Game Date** (lower 20 bits): Absolute encoding `((year << 9) | (month << 5) | day)`
- **Event Date** (upper 12 bits): Relative encoding (±3 years from game date)
- **No year offset**: Years stored directly (2022 = 2022, not 2022-1408)
- **Working implementation**: Verified with round-trip testing

### ✅ 1.3 Test Data Analysis - DONE
- **Confirmed test data issue**: All games in test database have same date (2022.12.19)
- **Pattern search was wrong approach**: Should read from fixed offset 25-28
- **Working implementation**: `experiments/scid_parser/src/main.rs`

---

## Phase 2: Fix Core Implementation

### 2.1 Complete Date Parsing Implementation ⏳
**Priority**: Critical  
**Effort**: 1-2 hours  
**Files**: `src/scid/index.rs` (lines 205-280)

**STATUS**: Partially implemented - game date extraction works, event date missing

- [x] **Fixed offset reading implemented** ✅ (line 212-225)
  - Date field correctly read from offset 25-28
  - Little-endian conversion working
  - Error handling for short entries

- [x] **Game date extraction works** ✅ (lines 252-262)
  - 20-bit game date extraction from lower bits
  - Correct bit field extraction (day, month, year)
  - No incorrect year offset (years stored directly)

- [ ] **Add event date extraction** (upper 12 bits)
  ```rust
  // After line 262, add event date extraction:
  
  // Extract event date from upper 12 bits (relative encoding)
  let event_data = (date_value >> 20) & 0xFFF;
  let event_date = if event_data == 0 {
      None
  } else {
      let event_day = (event_data & 31) as u8;
      let event_month = ((event_data >> 5) & 15) as u8;
      let year_offset = ((event_data >> 9) & 7) as u16;
      
      if year_offset == 0 {
          None
      } else {
          let event_year = actual_year + year_offset - 4;
          Some(ScidDate { year: event_year, month: event_month, day: event_day })
      }
  };
  
  println!("DEBUG: Event date: {:?}", event_date);
  ```

- [ ] **Update GameIndexEntry structure** to include event date
  ```rust
  // In src/scid/types.rs or wherever GameIndexEntry is defined:
  pub struct GameIndexEntry {
      // ... existing fields ...
      pub game_date: ScidDate,
      pub event_date: Option<ScidDate>,  // ADD THIS
      // ... rest of fields ...
  }
  ```

- [ ] **Clean up debug prints** (lines 220, 253, 255)
  - Replace with proper logging or remove entirely
  - Keep error logging for production debugging

### 2.2 Add Event Date Support ⏳
**Priority**: High  
**Effort**: 2-3 hours  
**Files**: `src/scid/index.rs`, `src/scid/types.rs`

- [ ] **Extend ScidDate structure** to include optional event date
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub struct ScidGameDates {
      pub game_date: ScidDate,
      pub event_date: Option<ScidDate>,
  }
  ```

- [ ] **Implement event date extraction**
  ```rust
  // Extract event date from upper 12 bits (relative encoding)
  let event_data = (dates_field >> 20) & 0xFFF;
  let event_date = if event_data == 0 {
      None
  } else {
      let event_day = (event_data & 31) as u8;
      let event_month = ((event_data >> 5) & 15) as u8;
      let year_offset = ((event_data >> 9) & 7) as u16;
      
      if year_offset == 0 {
          None
      } else {
          let event_year = year + year_offset - 4;
          Some(ScidDate { year: event_year, month: event_month, day: event_day })
      }
  };
  ```

### 2.3 Update PGN Output ⏳
**Priority**: High  
**Effort**: 1 hour  
**Files**: `src/pgn/exporter.rs` (line 104)

- [ ] **Update date output in PGN exporter**
  ```rust
  // CURRENT (line 104): 
  writeln!(writer, "[Date \"{}\"]", game_index.date_string())?;
  
  // UPDATE TO:
  writeln!(writer, "[Date \"{}\"]", game_index.game_date_string())?;
  
  // ADD EVENT DATE SUPPORT:
  if let Some(event_date_str) = game_index.event_date_string() {
      writeln!(writer, "[EventDate \"{}\"]", event_date_str)?;
  }
  ```

- [ ] **Update GameIndexEntry date methods**
  ```rust
  // Update or add methods to GameIndexEntry:
  impl GameIndexEntry {
      pub fn game_date_string(&self) -> String {
          format!("{}.{:02}.{:02}", self.game_date.year, self.game_date.month, self.game_date.day)
      }
      
      pub fn event_date_string(&self) -> Option<String> {
          self.event_date.as_ref().map(|date| 
              format!("{}.{:02}.{:02}", date.year, date.month, date.day)
          )
      }
      
      // Keep existing date_string() for backward compatibility
      pub fn date_string(&self) -> String {
          self.game_date_string()
      }
  }
  ```

- [ ] **Ensure backward compatibility**
  - `date_string()` method should still work for existing code
  - Date field should always be present in PGN output
  - EventDate field should only appear when event date is available

### 2.3 Error Handling and Validation ⏳
**Priority**: High  
**Effort**: 1-2 hours  

- [ ] **Add date validation**
  ```rust
  fn validate_date(date: &ScidDate) -> Result<(), ScidError> {
      if date.month < 1 || date.month > 12 {
          return Err(ScidError::InvalidDate("Invalid month"));
      }
      if date.day < 1 || date.day > 31 {
          return Err(ScidError::InvalidDate("Invalid day"));
      }
      if date.year > 2047 {  // SCID's 11-bit year limit
          return Err(ScidError::InvalidDate("Year exceeds SCID format limit"));
      }
      Ok(())
  }
  ```

- [ ] **Handle malformed date fields gracefully**
  - Return sensible defaults for invalid dates
  - Log warnings for suspicious date values
  - Continue processing other games if one has bad dates

---

## Phase 3: Testing and Validation

### 3.1 Update Unit Tests ⏳
**Priority**: High  
**Effort**: 2-3 hours  
**Files**: `tests/date_extraction_tests.rs`

- [ ] **Remove hardcoded pattern tests**
  ```rust
  // REMOVE: Tests that assume specific pattern values
  // REMOVE: Tests with hardcoded 0x0944cd93 pattern
  ```

- [ ] **Add fixed-offset date parsing tests**
  ```rust
  #[test]
  fn test_dates_field_extraction() {
      // Test reading dates from fixed offset 25-28
      let test_data = create_test_index_entry_with_dates(2020, 6, 15, Some((2020, 6, 10)));
      let dates = parse_game_dates(&test_data[25..29]).unwrap();
      
      assert_eq!(dates.game_date.year, 2020);
      assert_eq!(dates.game_date.month, 6);
      assert_eq!(dates.game_date.day, 15);
      
      assert!(dates.event_date.is_some());
      let event_date = dates.event_date.unwrap();
      assert_eq!(event_date.year, 2020);
      assert_eq!(event_date.month, 6);
      assert_eq!(event_date.day, 10);
  }
  ```

- [ ] **Add event date parsing tests**
  ```rust
  #[test]
  fn test_event_date_relative_encoding() {
      // Test various year offsets (±3 years)
      let test_cases = vec![
          (2020, 6, 15, 2022, 12, 19), // +2 years
          (2020, 6, 15, 2018, 3, 10),  // -2 years
          (2020, 6, 15, 2020, 6, 15),  // Same date
      ];
      
      for (gy, gm, gd, ey, em, ed) in test_cases {
          let dates_field = create_dates_field(gy, gm, gd, Some((ey, em, ed)));
          let dates = parse_dates_field(dates_field).unwrap();
          
          assert_eq!(dates.game_date, ScidDate::new(gy, gm, gd));
          assert_eq!(dates.event_date, Some(ScidDate::new(ey, em, ed)));
      }
  }
  ```

### 3.2 Integration Tests ⏳
**Priority**: High  
**Effort**: 1-2 hours  

- [ ] **Test with existing dataset** (five.pgn)
  ```rust
  #[test]
  fn test_existing_dataset_backward_compatibility() {
      // Ensure existing five.pgn dataset still produces correct dates
      // All games should still show 2022.12.19
      let pgn_output = convert_scid_to_pgn("tests/data/test.si4").unwrap();
      assert!(pgn_output.contains("[Date \"2022.12.19\"]"));
  }
  ```

- [ ] **Create test with multiple dates**
  ```rust
  #[test]
  fn test_multiple_date_scenarios() {
      // Test database with games from different dates
      // Verify each game shows its correct date
      // Use the working implementation from experiments/scid_parser as reference
  }
  ```

### 3.3 Reference Implementation Validation ⏳
**Priority**: Medium  
**Effort**: 1 hour  

- [ ] **Copy working code from experiments**
  - Use `experiments/scid_parser/src/main.rs` as reference
  - Port the proven `scid_get_event_date()` and date parsing logic
  - Ensure bit manipulation matches exactly

- [ ] **Cross-validate with SCID source**
  - Compare implementation against `scidvspc/src/index.cpp`
  - Verify bit field extraction matches C++ code
  - Test edge cases (no event date, year out of range)

---

## Phase 4: Documentation and Cleanup

### 4.1 Update Documentation ⏳
**Priority**: Medium  
**Effort**: 1 hour  

- [ ] **Update DATE_PARSING_ISSUE.md**
  - Mark issue as resolved
  - Document the actual solution
  - Explain the game vs event date distinction

- [ ] **Update CLAUDE.md**
  - Document the fix process and learnings
  - Note the importance of the SCID_DATABASE_FORMAT.md documentation
  - Add notes about testing methodology

### 4.2 Code Documentation ⏳
**Priority**: Low  
**Effort**: 30 minutes  

- [ ] **Add comprehensive comments** to date parsing code
- [ ] **Document the dual date structure** in code comments
- [ ] **Add examples** showing typical date field values

---

## Success Criteria

### ✅ **Primary Goals**
- [ ] Date parsing works for databases with different game dates
- [ ] Event date extraction works when available
- [ ] No hardcoded patterns or offsets (except fixed SCID field positions)
- [ ] All existing tests continue to pass

### ✅ **Secondary Goals**
- [ ] Performance equivalent to current implementation  
- [ ] Robust error handling for malformed date fields
- [ ] PGN output includes both game and event dates when available
- [ ] Clear code documentation

### ✅ **Validation Criteria**
- [ ] Working experiments/scid_parser implementation validates core logic
- [ ] Original test dataset still produces correct 2022.12.19 dates
- [ ] New test datasets with different dates show correct values
- [ ] Event dates are correctly extracted when present

---

## Implementation Notes

### **Key Technical Details**
- **Dates field location**: Always offset 25-28 in 47-byte index entry
- **Bit layout**: `[31-20: Event Date] [19-0: Game Date]`
- **Game date encoding**: `((year << 9) | (month << 5) | day)` - absolute
- **Event date encoding**: Relative to game date (±3 years max)
- **Event date year offset**: `stored_offset = (event_year - game_year + 4) & 7`

### **Reference Implementation**
- Working code in: `experiments/scid_parser/src/main.rs`
- Functions: `scid_set_event_date()`, `scid_get_event_date()`, `date_make()`, etc.
- Tested with: Round-trip encoding/decoding validation

### **Critical Findings**
- **NO year offset needed**: Years stored directly (not year - 1900 or year - 1408)
- **Pattern search was wrong**: Should use fixed offset, not search for patterns
- **Dual date support**: SCID format includes both game and event dates
- **Little-endian encoding**: All multi-byte values in SCID are little-endian

---

## 🚀 Quick Start Implementation Guide

### **Immediate Next Steps** (1-2 hours)

1. **Add event date extraction** to `src/scid/index.rs` around line 262:
   ```rust
   // After the game date extraction, add:
   let event_data = (date_value >> 20) & 0xFFF;
   let event_date = if event_data == 0 {
       None
   } else {
       let event_day = (event_data & 31) as u8;
       let event_month = ((event_data >> 5) & 15) as u8; 
       let year_offset = ((event_data >> 9) & 7) as u16;
       if year_offset == 0 { None } else {
           Some(ScidDate { year: actual_year + year_offset - 4, month: event_month, day: event_day })
       }
   };
   ```

2. **Update GameIndexEntry structure** to include `event_date: Option<ScidDate>`

3. **Test with existing dataset** - should still produce 2022.12.19 for all games

4. **Update PGN output** in `src/pgn/exporter.rs` line 104 to include EventDate header

### **Validation Steps** (30 minutes)

1. **Run existing tests** - ensure no regression
2. **Test with experiments/scid_parser** - compare output for same database  
3. **Check PGN format** - ensure both Date and EventDate headers are correct

### **Copy-Paste Reference Code**

Use the working implementations from `experiments/scid_parser/src/main.rs`:
- `scid_get_event_date()` function (lines 361-381)
- Date parsing logic (lines 258-270) 
- Round-trip testing approach (lines 429-457)

The research is complete - implementation should be straightforward! 🎯  

- [x] **Research correct year offset formula** ✅ **COMPLETED**
  - **FINDING**: Official SCID source code has NO year offset
  - Years are stored directly as actual values (2022 stored as 2022) 
  - The +1408 offset is completely incorrect and not part of SCID spec

- [ ] **Remove hardcoded +1408 offset** (Line 276)
  ```rust
  // REMOVE: let year = year_raw + 1408;
  // REPLACE WITH: let year = year_raw; // Direct year value
  ```

- [ ] **Update bit-field extraction to match SCID spec**
  ```rust
  // Official SCID encoding: DATE_MAKE(year, month, day)
  // year << 9 | month << 5 | day
  let day = (date_pattern & 31) as u8;           // Bits 0-4 (5 bits)
  let month = ((date_pattern >> 5) & 15) as u8;  // Bits 5-8 (4 bits) 
  let year = ((date_pattern >> 9) & 0x7FF) as u16; // Bits 9-19 (11 bits)
  // No offset needed - year is stored directly
  ```

- [ ] **Add bounds checking for year values**
  - Validate year is reasonable (e.g., 1500-2047, SCID's max year)
  - Handle edge cases gracefully
  - Log warnings for years outside expected range

---

## Phase 3: Testing and Validation

### 3.1 Update Unit Tests ⏳
**Priority**: High  
**Effort**: 2-3 hours  

**Files**: `src/scid/index.rs`, `tests/*`

- [ ] **Update existing unit tests**
  - Remove tests that assume hardcoded pattern
  - Add tests for different date values
  - Test edge cases (invalid dates, boundary values)

- [ ] **Add tests for multiple date patterns**
  ```rust
  #[test]
  fn test_various_date_patterns() {
      let test_cases = vec![
          (2020, 3, 15, expected_pattern_1),
          (2021, 7, 22, expected_pattern_2),
          (2023, 11, 30, expected_pattern_3),
      ];
      // Test each pattern decodes correctly
  }
  ```

- [ ] **Test year offset edge cases**
  - Very old games (1990s)
  - Very new games (2030s)
  - Games at year boundaries

### 3.2 Create Integration Tests ⏳
**Priority**: High  
**Effort**: 2-3 hours  

**Files**: `tests/`

- [ ] **Test with multi-date database**
  ```rust
  #[test]
  fn test_multiple_dates_database() {
      // Load database with games from different dates
      // Verify each game shows correct date
      // Compare against PGN source of truth
  }
  ```

- [ ] **Test backward compatibility**
  - Ensure original test dataset still works
  - Verify five.pgn dates still match
  - No regression in existing functionality

- [ ] **Test error handling**
  - Malformed date fields
  - Corrupted game indices
  - Unexpected patterns

### 3.3 Validation Against Real Data ⏳
**Priority**: High  
**Effort**: 2-4 hours  

- [ ] **Test with real SCID databases**
  - Download or create databases with known dates
  - Run conversion and verify PGN output
  - Compare dates against chess database websites

- [ ] **Cross-validate with official SCID tools**
  - Export same database with official SCID
  - Compare date fields in resulting PGN
  - Ensure our implementation matches official output

- [ ] **Performance testing**
  - Test with large databases (10K+ games)
  - Verify no performance regression
  - Ensure memory usage remains reasonable

---

## Phase 4: Documentation and Cleanup

### 4.1 Update Documentation ⏳
**Priority**: Medium  
**Effort**: 1 hour  

**Files**: `CLAUDE.md`, `DATE_PARSING_ISSUE.md`

- [ ] **Update CLAUDE.md**
  - Remove "FULLY RESOLVED" claims until actually fixed
  - Document the fix process and lessons learned
  - Add notes about testing with multiple date databases

- [ ] **Document the final solution**
  - Explain correct date field offset and structure
  - Document year offset calculation method
  - Add troubleshooting guide for date issues

### 4.2 Code Cleanup ⏳
**Priority**: Low  
**Effort**: 30 minutes  

- [ ] **Remove debug prints** related to pattern searching
- [ ] **Add comprehensive code comments** explaining date parsing
- [ ] **Update function documentation** with correct behavior
- [ ] **Clean up unused constants** and variables

---

## Phase 5: Quality Assurance

### 5.1 Comprehensive Testing ⏳
**Priority**: High  
**Effort**: 2-3 hours  

- [ ] **Run full test suite** and ensure all tests pass
- [ ] **Test with various SCID databases**
  - Tournament databases
  - Historical game collections  
  - Personal game databases
  
- [ ] **Validate PGN output**
  - Import generated PGN into chess software
  - Verify dates display correctly
  - Check for any format issues

### 5.2 Code Review Preparation ⏳
**Priority**: Medium  
**Effort**: 1 hour  

- [ ] **Self-review all changes**
  - Ensure no hardcoded values remain
  - Verify error handling is robust
  - Check for potential edge cases

- [ ] **Performance verification**
  - Benchmark before/after performance
  - Ensure no memory leaks
  - Verify reasonable processing speed

---

## Success Criteria

### ✅ **Primary Goals**
- [ ] Date parsing works for databases with different dates
- [ ] No hardcoded date patterns in production code
- [ ] All existing tests continue to pass
- [ ] New tests validate multiple date scenarios

### ✅ **Secondary Goals**  
- [ ] Performance equivalent to current implementation
- [ ] Robust error handling for malformed data
- [ ] Clear documentation of date parsing approach
- [ ] Easy to maintain and extend

### ✅ **Validation Criteria**
- [ ] Test database with 2020.03.15 shows correct date
- [ ] Test database with 2023.11.30 shows correct date  
- [ ] Original five.pgn dataset still shows 2022.12.19
- [ ] Generated PGN files import correctly into chess software

---

## Risk Mitigation

### 🔴 **High Risks**
- **SCID format variations**: Different SCID versions may use different structures
  - *Mitigation*: Test with multiple SCID versions and databases
  
- **Year offset assumptions**: +1408 offset may be specific to test data
  - *Mitigation*: Research SCID source code thoroughly, test with various year ranges

### 🟡 **Medium Risks**  
- **Breaking existing functionality**: Changes may affect other parts of system
  - *Mitigation*: Comprehensive regression testing
  
- **Performance impact**: New implementation may be slower
  - *Mitigation*: Benchmark and optimize if needed

### 🟢 **Low Risks**
- **Test data availability**: May be hard to find databases with specific dates
  - *Mitigation*: Create synthetic test databases if needed

---

## Notes for Future Developer

### **Key Insights**
- The original issue was masked by test data that all had the same date
- Pattern searching was the wrong approach - should use fixed field offsets
- SCID format is binary and has fixed structure - leverage this
- Year offset calculation may need to be calibrated per SCID version

### **Files to Focus On**
- `src/scid/index.rs` - Primary implementation file
- `tests/date_extraction_tests.rs` - Core test validation
- `DATE_PARSING_ISSUE.md` - Historical context and problem analysis

### **Testing Strategy**
- Always test with databases containing multiple different dates
- Validate against PGN exports from official SCID tools
- Don't rely solely on unit tests - integration testing is crucial

### **Debugging Tips**
- Use hex dumps to analyze binary date field structure
- Cross-reference with SCID source code at https://github.com/nloding/scidvspc
- Test with both old and new SCID database versions