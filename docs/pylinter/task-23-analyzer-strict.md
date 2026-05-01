# Task 23: Analyzer — strict level (code quality)

> Status: ⬜ Not started
> Depends: Task 05
> Output: strict level rules — unnecessary pass, empty f-string, redundant if-return

## Goal

Impl analyzer strict level — 3 rules:

| Rule | rule_id | Severity |
|---|---|---|
| Unnecessary pass | `readability-unnecessary-pass` | Warning |
| f-string no placeholder | `readability-empty-fstring` | Warning |
| Redundant nested if-return | `readability-simplify-if-return` | Warning |

## Ref

- `cclinter/src/analyzer/strict.rs` — structure ref

## Steps

### 1. analyzer/strict.rs

```rust
pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_unnecessary_pass(source));
    diags.extend(check_empty_fstring(source));
    diags.extend(check_simplify_if_return(source));
    diags
}
```

### 2. Unnecessary pass

```rust
fn check_unnecessary_pass(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk FunctionDef/ClassDef/If/For/While/Try etc
    // body has pass + body.len() > 1 → report
    // body has docstring + pass → pass unnecessary → report
    // Exclude: body is only pass (required)
}
```

```python
def foo():
    """Docstring."""
    pass  # Warning: unnecessary pass
```

### 3. Empty f-string

```rust
fn check_empty_fstring(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk JoinedStr (f-string) nodes
    // No FormattedValue inside → report
    // f"hello" flagged, f"hello {name}" OK
}
```

### 4. Redundant if-return

```rust
fn check_simplify_if_return(source: &SourceFile) -> Vec<Diagnostic> {
    // Pattern: if cond: return True else: return False → return cond
    // Pattern: if cond: return expr → return cond and expr (contextual)
    // Start simple: detect if x: return True / return False
}
```

## Tests

```rust
#[test]
fn unnecessary_pass_with_docstring() {
    let src = "def foo():\n    \"\"\"Doc.\"\"\"\n    pass\n";
    // Expect report
}

#[test]
fn pass_only_ok() {
    let src = "def foo():\n    pass\n";
    // No report (pass required)
}

#[test]
fn empty_fstring() {
    let src = "x = f\"hello\"\n";
    // Expect readability-empty-fstring
}

#[test]
fn fstring_with_placeholder_ok() {
    let src = "x = f\"hello {name}\"\n";
    // No report
}

#[test]
fn simplify_if_return_bool() {
    let src = "def foo(x):\n    if x:\n        return True\n    else:\n        return False\n";
    // Expect readability-simplify-if-return
}

#[test]
fn normal_if_return_ok() {
    let src = "def foo(x):\n    if x:\n        return compute()\n    return 0\n";
    // No report
}
```

## Verify

```bash
cargo test -- analyzer::strict
```
