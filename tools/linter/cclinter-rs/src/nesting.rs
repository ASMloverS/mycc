use std::sync::LazyLock;

use regex::Regex;

use crate::error::{ErrorCategory, Violation};

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmState {
    NoAsm,
    InsideAsm,
    EndAsm,
    BlockAsm,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub starting_linenum: usize,
    pub seen_open_brace: bool,
    pub open_parentheses: usize,
    pub inline_asm: AsmState,
    pub name: String,
    pub access: String,
    pub is_derived: bool,
    pub is_struct: bool,
    pub class_indent: usize,
    pub last_line: usize,
}

#[derive(Debug, Clone)]
pub struct NamespaceInfo {
    pub starting_linenum: usize,
    pub seen_open_brace: bool,
    pub open_parentheses: usize,
    pub inline_asm: AsmState,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ExternCInfo {
    pub starting_linenum: usize,
    pub seen_open_brace: bool,
    pub open_parentheses: usize,
    pub inline_asm: AsmState,
}

#[derive(Debug, Clone)]
pub enum BlockKind {
    Class(ClassInfo),
    Namespace(NamespaceInfo),
    ExternC(ExternCInfo),
}

#[derive(Debug, Clone)]
pub struct PreprocessorSnapshot {
    pub stack: Vec<BlockKind>,
    pub stack_before_else: Vec<BlockKind>,
    pub seen_else: bool,
}

#[derive(Debug, Clone)]
pub struct NestingState {
    pub stack: Vec<BlockKind>,
    pub pp_stack: Vec<PreprocessorSnapshot>,
    pub previous_stack_top: Option<BlockKind>,
}

// ── Static regexes ─────────────────────────────────────────────────────

static CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:class|struct)\s+(\w+)").unwrap());

static NAMESPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"namespace\s+(\w+)").unwrap());

static ASM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:asm|__asm__)\s*(?:volatile\s*)?[{(]").unwrap());

// ── BlockKind accessors ────────────────────────────────────────────────

impl BlockKind {
    pub fn starting_linenum(&self) -> usize {
        match self {
            BlockKind::Class(c) => c.starting_linenum,
            BlockKind::Namespace(n) => n.starting_linenum,
            BlockKind::ExternC(e) => e.starting_linenum,
        }
    }

    pub fn seen_open_brace(&self) -> bool {
        match self {
            BlockKind::Class(c) => c.seen_open_brace,
            BlockKind::Namespace(n) => n.seen_open_brace,
            BlockKind::ExternC(e) => e.seen_open_brace,
        }
    }

    pub fn set_seen_open_brace(&mut self, v: bool) {
        match self {
            BlockKind::Class(c) => c.seen_open_brace = v,
            BlockKind::Namespace(n) => n.seen_open_brace = v,
            BlockKind::ExternC(e) => e.seen_open_brace = v,
        }
    }

    pub fn open_parentheses(&self) -> usize {
        match self {
            BlockKind::Class(c) => c.open_parentheses,
            BlockKind::Namespace(n) => n.open_parentheses,
            BlockKind::ExternC(e) => e.open_parentheses,
        }
    }

    pub fn set_open_parentheses(&mut self, count: usize) {
        match self {
            BlockKind::Class(c) => c.open_parentheses = count,
            BlockKind::Namespace(ns) => ns.open_parentheses = count,
            BlockKind::ExternC(e) => e.open_parentheses = count,
        }
    }

    pub fn inline_asm(&self) -> &AsmState {
        match self {
            BlockKind::Class(c) => &c.inline_asm,
            BlockKind::Namespace(n) => &n.inline_asm,
            BlockKind::ExternC(e) => &e.inline_asm,
        }
    }

    pub fn set_inline_asm(&mut self, s: AsmState) {
        match self {
            BlockKind::Class(c) => c.inline_asm = s,
            BlockKind::Namespace(n) => n.inline_asm = s,
            BlockKind::ExternC(e) => e.inline_asm = s,
        }
    }

    pub fn is_class(&self) -> bool {
        matches!(self, BlockKind::Class(_))
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, BlockKind::Namespace(_))
    }

    pub fn is_extern_c(&self) -> bool {
        matches!(self, BlockKind::ExternC(_))
    }
}

// ── NestingState impl ──────────────────────────────────────────────────

impl Default for NestingState {
    fn default() -> Self {
        Self::new()
    }
}

impl NestingState {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            pp_stack: Vec::new(),
            previous_stack_top: None,
        }
    }

    pub fn update(&mut self, line: &str, linenum: usize) {
        self.previous_stack_top = self.stack.last().cloned();

        let trimmed = line.trim();

        // Skip full-line comments
        if trimmed.starts_with("//") {
            return;
        }

        // ── Preprocessor directives ──
        if let Some(stripped) = trimmed.strip_prefix('#') {
            let directive = stripped.trim_start();
            if directive.starts_with("if") {
                // #if, #ifdef, #ifndef
                self.pp_stack.push(PreprocessorSnapshot {
                    stack: self.stack.clone(),
                    stack_before_else: Vec::new(),
                    seen_else: false,
                });
            } else if directive.starts_with("else") || directive.starts_with("elif") {
                if let Some(snapshot) = self.pp_stack.last_mut() {
                    if !snapshot.seen_else {
                        snapshot.stack_before_else = self.stack.clone();
                        snapshot.seen_else = true;
                    }
                    self.stack = snapshot.stack.clone();
                }
            } else if directive.starts_with("endif") {
                if let Some(snapshot) = self.pp_stack.pop() {
                    if snapshot.seen_else {
                        self.stack = snapshot.stack_before_else;
                    } else {
                        self.stack = snapshot.stack;
                    }
                }
            }
            // Process the rest of the line for brace tracking even in preprocessor lines
            self.update_braces(trimmed);
            return;
        }

        // ── Extern "C" detection ──
        if let Some(pos) = find_extern_c(trimmed) {
            let rest = &trimmed[pos..];
            if rest.contains('{') {
                let info = ExternCInfo {
                    starting_linenum: linenum,
                    seen_open_brace: true,
                    open_parentheses: 0,
                    inline_asm: AsmState::NoAsm,
                };
                let mut brace_count: i32 = 1;
                for ch in rest.chars() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => brace_count -= 1,
                        _ => {}
                    }
                }
                if brace_count > 0 {
                    self.stack.push(BlockKind::ExternC(info));
                }
                // Count braces/parens on rest for any existing stack entries
                self.update_braces_from(rest);
                return;
            }
        }

        // ── Class/struct detection ──
        if let Some(caps) = CLASS_RE.captures(trimmed) {
            let name = caps[1].to_string();
            let is_struct = trimmed.starts_with("struct");
            let is_derived = trimmed.contains(':');
            let class_indent = line.len() - line.trim_start().len();

            let mut info = ClassInfo {
                starting_linenum: linenum,
                seen_open_brace: false,
                open_parentheses: 0,
                inline_asm: AsmState::NoAsm,
                name,
                access: if is_struct {
                    "public".to_string()
                } else {
                    "private".to_string()
                },
                is_derived,
                is_struct,
                class_indent,
                last_line: linenum,
            };

            // Check if the line itself has an opening brace
            let after_match = &trimmed[caps.get(0).unwrap().end()..];
            if after_match.contains('{') {
                info.seen_open_brace = true;
            }

            self.stack.push(BlockKind::Class(info));
            self.update_braces(trimmed);
            return;
        }

        // ── Namespace detection ──
        if let Some(caps) = NAMESPACE_RE.captures(trimmed) {
            let name = caps[1].to_string();
            let mut info = NamespaceInfo {
                starting_linenum: linenum,
                seen_open_brace: false,
                open_parentheses: 0,
                inline_asm: AsmState::NoAsm,
                name,
            };

            let after_match = &trimmed[caps.get(0).unwrap().end()..];
            if after_match.contains('{') {
                info.seen_open_brace = true;
            }

            self.stack.push(BlockKind::Namespace(info));
            self.update_braces(trimmed);
            return;
        }

        // ── ASM detection on stack top ──
        if ASM_RE.is_match(trimmed) {
            if let Some(top) = self.stack.last_mut() {
                top.set_inline_asm(AsmState::InsideAsm);
            }
        }

        self.update_braces(trimmed);
    }

    fn update_braces(&mut self, line: &str) {
        self.update_braces_from(line);
    }

    fn update_braces_from(&mut self, line: &str) {
        if self.stack.is_empty() {
            return;
        }

        let mut paren_depth: usize = 0;
        let mut brace_delta: i32 = 0;

        // Simple character scan skipping string/char literals
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\'' => {
                    // skip char literal
                    while let Some(c) = chars.next() {
                        if c == '\\' {
                            chars.next();
                        } else if c == '\'' {
                            break;
                        }
                    }
                }
                '"' => {
                    // skip string literal
                    while let Some(c) = chars.next() {
                        if c == '\\' {
                            chars.next();
                        } else if c == '"' {
                            break;
                        }
                    }
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        break; // rest is comment
                    }
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        // skip block comment
                        while let Some(c) = chars.next() {
                            if c == '*' && chars.peek() == Some(&'/') {
                                chars.next();
                                break;
                            }
                        }
                    }
                }
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth = paren_depth.saturating_sub(1);
                }
                '{' => brace_delta += 1,
                '}' => brace_delta -= 1,
                _ => {}
            }
        }

        let top = match self.stack.last_mut() {
            Some(t) => t,
            None => return,
        };

        // Track parentheses
        let new_paren = top.open_parentheses() as i32 + paren_depth as i32;
        top.set_open_parentheses(if new_paren < 0 {
            0
        } else {
            new_paren as usize
        });

        // If we haven't seen the opening brace yet, just check for it
        if !top.seen_open_brace() && brace_delta > 0 {
            top.set_seen_open_brace(true);
            brace_delta -= 1;
            // apply remaining brace_delta
        }

        // Apply brace_delta: if we go to 0 or below after having seen open brace, pop
        if brace_delta < 0 && top.seen_open_brace() {
            // The closing brace balances the block — pop it
            self.stack.pop();
        }
    }

    pub fn innermost_class(&self) -> Option<&ClassInfo> {
        self.stack.iter().rev().find_map(|bk| match bk {
            BlockKind::Class(c) => Some(c),
            _ => None,
        })
    }

    pub fn innermost_namespace(&self) -> Option<&NamespaceInfo> {
        self.stack.iter().rev().find_map(|bk| match bk {
            BlockKind::Namespace(n) => Some(n),
            _ => None,
        })
    }

    pub fn in_class(&self) -> bool {
        self.stack.iter().any(|bk| bk.is_class())
    }

    pub fn in_namespace(&self) -> bool {
        self.stack.iter().any(|bk| bk.is_namespace())
    }

    pub fn in_extern_c(&self) -> bool {
        self.stack.iter().any(|bk| bk.is_extern_c())
    }

    pub fn in_asm_block(&self) -> bool {
        self.stack
            .iter()
            .any(|bk| *bk.inline_asm() == AsmState::InsideAsm)
    }

    pub fn stack_top(&self) -> Option<&BlockKind> {
        self.stack.last()
    }

    pub fn previous_stack_top(&self) -> Option<&BlockKind> {
        self.previous_stack_top.as_ref()
    }

    pub fn check_completed_blocks(&self, filename: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        for bk in &self.stack {
            let msg = match bk {
                BlockKind::Class(c) => {
                    format!("Unmatched {{ for class/struct {} at line {}", c.name, c.starting_linenum)
                }
                BlockKind::Namespace(n) => {
                    format!(
                        "Unmatched {{ for namespace {} at line {}",
                        n.name, n.starting_linenum
                    )
                }
                BlockKind::ExternC(e) => {
                    format!("Unmatched {{ for extern \"C\" at line {}", e.starting_linenum)
                }
            };
            violations.push(Violation {
                filename: filename.to_string(),
                linenum: bk.starting_linenum(),
                category: ErrorCategory::ReadabilityBraces,
                confidence: 80,
                message: msg,
            });
        }
        violations
    }
}

/// Find the position after `extern "C"` or `extern 'C'` in the line.
/// Returns the end position of the match, or None.
fn find_extern_c(line: &str) -> Option<usize> {
    let re: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"extern\s+"C""#).unwrap());
    re.find(line).map(|m| m.end())
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state() {
        let state = NestingState::new();
        assert!(state.stack.is_empty());
        assert!(state.pp_stack.is_empty());
        assert!(state.stack_top().is_none());
        assert!(state.previous_stack_top().is_none());
        assert!(!state.in_class());
        assert!(!state.in_namespace());
        assert!(!state.in_extern_c());
        assert!(!state.in_asm_block());
    }

    #[test]
    fn test_class_detect() {
        let mut state = NestingState::new();
        state.update("class Foo {", 1);
        assert!(state.in_class());
        assert!(state.stack_top().unwrap().is_class());
        let ci = state.innermost_class().unwrap();
        assert_eq!(ci.name, "Foo");
        assert!(!ci.is_struct);
        assert!(!ci.is_derived);
        assert!(ci.seen_open_brace);
    }

    #[test]
    fn test_class_close() {
        let mut state = NestingState::new();
        state.update("class Foo {", 1);
        assert!(state.in_class());
        state.update("  int x;", 2);
        assert!(state.in_class());
        state.update("};", 3);
        assert!(!state.in_class());
        assert!(state.stack.is_empty());
    }

    #[test]
    fn test_namespace_detect() {
        let mut state = NestingState::new();
        state.update("namespace myns {", 1);
        assert!(state.in_namespace());
        let ni = state.innermost_namespace().unwrap();
        assert_eq!(ni.name, "myns");
        assert!(ni.seen_open_brace);
    }

    #[test]
    fn test_extern_c() {
        let mut state = NestingState::new();
        state.update("extern \"C\" {", 1);
        assert!(state.in_extern_c());
        assert!(state.stack_top().unwrap().is_extern_c());
        state.update("}", 2);
        assert!(!state.in_extern_c());
    }

    #[test]
    fn test_nested_class_namespace() {
        let mut state = NestingState::new();
        state.update("namespace outer {", 1);
        state.update("  class Inner {", 2);
        assert!(state.in_namespace());
        assert!(state.in_class());
        assert_eq!(state.stack.len(), 2);
        assert_eq!(state.innermost_namespace().unwrap().name, "outer");
        assert_eq!(state.innermost_class().unwrap().name, "Inner");
    }

    #[test]
    fn test_preprocessor_save_restore() {
        let mut state = NestingState::new();
        state.update("class Foo {", 1);
        state.update("#ifdef SOMETHING", 2);
        // Snapshot should have the stack with Foo
        assert_eq!(state.pp_stack.len(), 1);
        state.update("class Bar {", 3);
        assert_eq!(state.stack.len(), 2);
        state.update("#endif", 4);
        // Should restore to just Foo
        assert_eq!(state.stack.len(), 1);
        assert!(state.stack[0].is_class());
    }

    #[test]
    fn test_previous_stack_top() {
        let mut state = NestingState::new();
        state.update("class Foo {", 1);
        state.update("  int x;", 2);
        // After the second update, previous_stack_top should be the Foo block
        let prev = state.previous_stack_top().unwrap();
        assert!(prev.is_class());
    }

    #[test]
    fn test_struct_detect() {
        let mut state = NestingState::new();
        state.update("struct Point {", 5);
        assert!(state.in_class());
        let ci = state.innermost_class().unwrap();
        assert_eq!(ci.name, "Point");
        assert!(ci.is_struct);
        assert_eq!(ci.access, "public");
    }

    #[test]
    fn test_asm_block() {
        let mut state = NestingState::new();
        state.update("void foo() {", 1);
        // No block tracking for bare braces without a class/namespace/extern
        // ASM detection only works on stack top
        state.update("class Foo {", 2);
        state.update("  asm {", 3);
        assert!(state.in_asm_block());
    }
}
