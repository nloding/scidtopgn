
You are implementing a Rust library that converts SCID chess database files (.si4, .sn4, .sg4) to PGN format.

### Project Context

**What is SCID?** SCID (Shane's Chess Information Database) is a chess database format using three binary files:
- `.si4` - Index file (header + 47-byte game entries)
- `.sn4` - Name file (players, events, sites, rounds with front-coding compression)
- `.sg4` - Game file (tags, moves, comments, variations)

**Goal**: Create a Rust library and CLI tool that reads these files and outputs valid PGN.

**Project Structure**:
```
scidtopgn/
├── crates/
│   ├── core/          # Library crate (scidtopgn-core)
│   └── cli/           # Binary crate (scidtopgn)
├── tests/data/        # Test data: one.* (1 game) and five.* (5 games)
│                      # Each has .pgn source + .si4/.sn4/.sg4 SCID files
├── IMPLEMENTATION_GUIDE.md   # Task checklist with line numbers
└── PHASE_*.md         # Detailed specifications for each phase
```

### Your Task

**Complete this task**: Task 5.1.3 - Chess960 Support

**How to find the specification**:
1. Open `IMPLEMENTATION_GUIDE.md` to find the task's phase document and line number
2. Read ONLY the relevant task section in the phase document
3. Implement exactly what the specification describes

### Coding Guidelines (MUST FOLLOW)

**From `.ai/RUST_INSTRUCTIONS.md`**:
- Use `Result<T, E>` for errors, never `unwrap()` or `expect()` in library code
- Prefer borrowing (`&T`) over cloning
- Use iterators instead of index-based loops
- Implement common traits: `Debug`, `Clone`, `PartialEq` where appropriate
- All public items need rustdoc comments (`///`)
- Code must pass `cargo fmt`, `cargo clippy`, and `cargo test`

**From `.ai/CODE_COMMENTING_INSTRUCTIONS.md`**:
- Write self-documenting code; comments explain WHY, not WHAT
- No obvious comments ("increment counter by one")
- No commented-out code or changelog comments
- DO comment: complex algorithms, regex patterns, external API constraints

**From `.ai/SENSIBLE_AI.md`**:
- Make minimal necessary changes
- Preserve existing code structure and style
- Don't add features not requested
- Don't refactor untouched code

### Implementation Rules

1. **Read the specification first** - Don't guess; the phase document has exact code
2. **Create only the files specified** - Don't add extra modules or helpers
3. **Match the struct/function signatures exactly** - They're designed to work together
4. **Include the tests from the spec** - They verify correctness
5. **Don't over-engineer** - Implement exactly what's asked, nothing more

### Verification

After implementing, run:
```bash
cargo build                    # Must compile without errors
cargo clippy                   # Must pass without warnings
cargo fmt -- --check           # Must be formatted
cargo test                     # All tests must pass
```

### Output Format

1. State which files you will create/modify
2. Implement the code
3. Run verification commands
4. Report results

If you encounter issues:
- Check if dependencies from earlier tasks are missing
- Read error messages carefully - they often indicate the fix
- Don't modify unrelated code to "fix" issues

---
