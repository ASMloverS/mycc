use std::sync::LazyLock;
use regex::Regex;

pub struct StackFrame {
    pub file: String,
    pub line: usize,
}

static GDB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"#\d+\s+.*\s+at\s+(.+):(\d+)").unwrap()
});
static MSVC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(.+)\((\d+)\):\s").unwrap()
});
static ADDR2LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\S+)\s+at\s+(.+):(\d+)").unwrap()
});
static BT_SYMBOLS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r".+\(.+\+.+\)\s+\[0x[0-9a-fA-F]+\]").unwrap()
});

pub fn parse_cpp_stacktrace(input: &str) -> Vec<StackFrame> {
    let mut frames = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(caps) = GDB_RE.captures(trimmed) {
            if let (Some(file), Some(line_s)) = (caps.get(1), caps.get(2)) {
                if let Ok(ln) = line_s.as_str().parse::<usize>() {
                    frames.push(StackFrame { file: file.as_str().to_string(), line: ln });
                    continue;
                }
            }
        }

        if let Some(caps) = MSVC_RE.captures(trimmed) {
            if let (Some(file), Some(line_s)) = (caps.get(1), caps.get(2)) {
                if let Ok(ln) = line_s.as_str().parse::<usize>() {
                    frames.push(StackFrame { file: file.as_str().to_string(), line: ln });
                    continue;
                }
            }
        }

        if let Some(caps) = ADDR2LINE_RE.captures(trimmed) {
            if let (Some(_func), Some(file), Some(line_s)) =
                (caps.get(1), caps.get(2), caps.get(3))
            {
                if let Ok(ln) = line_s.as_str().parse::<usize>() {
                    frames.push(StackFrame { file: file.as_str().to_string(), line: ln });
                    continue;
                }
            }
        }

        if BT_SYMBOLS_RE.is_match(trimmed) {
            crate::util::warn(
                &format!("skipping frame without file:line info: {}", trimmed),
                false,
            );
        }
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gdb_format() {
        let input = "#0  foo () at example.c:42\n#1  bar () at main.c:10";
        let frames = parse_cpp_stacktrace(input);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].file, "example.c");
        assert_eq!(frames[0].line, 42);
        assert_eq!(frames[1].file, "main.c");
        assert_eq!(frames[1].line, 10);
    }

    #[test]
    fn test_parse_msvc_format() {
        let input = "example.cpp(42): foo()\nmain.cpp(10): bar()";
        let frames = parse_cpp_stacktrace(input);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].file, "example.cpp");
        assert_eq!(frames[0].line, 42);
        assert_eq!(frames[1].file, "main.cpp");
        assert_eq!(frames[1].line, 10);
    }

    #[test]
    fn test_parse_addr2line_format() {
        let input = "foo at /path/example.c:42";
        let frames = parse_cpp_stacktrace(input);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].file, "/path/example.c");
        assert_eq!(frames[0].line, 42);
    }

    #[test]
    fn test_skip_backtrace_symbols() {
        let input = "./prog(foo+0x1a) [0x4005e6]";
        let frames = parse_cpp_stacktrace(input);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_parse_empty() {
        let frames = parse_cpp_stacktrace("");
        assert!(frames.is_empty());
    }

    #[test]
    fn test_parse_mixed() {
        let input = "#0  crash () at crash.c:100\n  ./app(handler+0x5) [0x401234]\n#1  main () at main.c:20";
        let frames = parse_cpp_stacktrace(input);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].file, "crash.c");
        assert_eq!(frames[1].file, "main.c");
    }
}
