# Daily Development Routine

## 🌅 Start of Day
1. Read master claude.md for current status
2. Check which phase we're in
3. Identify today's tasks
4. Run test suite to ensure clean state
```bash
cargo test --workspace
```

## 🔄 Development Cycle (Repeat)
1. Pick next task from current phase
2. Find relevant claude.md for context
3. Write failing test in appropriate test file
4. Run test - MUST see failure
5. Write MINIMAL implementation
6. Run test - MUST see pass
7. Refactor if needed (tests still pass)
8. Update relevant claude.md files
9. Commit with conventional message

## 📝 Commit Message Format
- `test: add failing test for X`
- `feat: implement minimal X to pass test`
- `refactor: clean up X implementation`
- `docs: update progress in claude.md`

## 🌙 End of Day
1. Run full quality gates
2. Update master claude.md progress
3. Note any blockers
4. Push all commits