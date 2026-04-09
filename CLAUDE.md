# CLAUDE.md

## Edit Protocol

### Read
- Grep target → Read w/ offset/limit (≤300 lines).
- Line 1 only if full survey needed.
- ±20 lines around target pre-edit.

### Write
- ≤100 lines/Edit.
- Larger → Edit-Verify cycle:
  1. Sub-change (≤100 lines).
  2. Syntax check → Language Checks below.
  3. Repeat.
- 1000+ line rename → `.patch`/`sed`.

### Lang Checks
- C/C++: `g++ -fsyntax-only <file>` / `cmake --build build`
- Rust: `cargo check`
- Go: `go build ./...` / `go vet ./...`
- Python: `python -m py_compile <file>` / `ruff check`
- TS/JS: `npx tsc --noEmit` / `npm run lint`
- Java: `mvn compile` / `gradle compileJava`
- C#: `dotnet build`
- Swift: `swift build`
- Kotlin: `kotlinc <file>` / `gradle compileKotlin`
- Ruby: `ruby -c <file>`
- Shell: `shellcheck <file>`
- Zig: `zig build`
- Match project toolchain. Skip if none.

### Forbidden
- 1 fn per Edit.
- Grep/Read before overwrite.

## Git
- No `Co-Authored-By: Claude ...` in commits.
