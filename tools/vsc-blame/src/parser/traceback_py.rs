use std::sync::LazyLock;
use regex::Regex;

pub struct TracebackFrame {
    pub file: String,
    pub line: usize,
}

static PY_FRAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"File "(.+?)", line (\d+)"#).unwrap()
});

pub fn parse_python_traceback(input: &str) -> Vec<TracebackFrame> {
    PY_FRAME_RE
        .find_iter(input)
        .filter_map(|m| {
            let caps = PY_FRAME_RE.captures(m.as_str())?;
            Some(TracebackFrame {
                file: caps[1].to_string(),
                line: caps[2].parse().ok()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_traceback() {
        let input = r#"Traceback (most recent call last):
  File "example.py", line 42, in <module>
    foo()
  File "example.py", line 10, in foo
    bar()"#;
        let frames = parse_python_traceback(input);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].file, "example.py");
        assert_eq!(frames[0].line, 42);
        assert_eq!(frames[1].file, "example.py");
        assert_eq!(frames[1].line, 10);
    }

    #[test]
    fn test_parse_chained_exception() {
        let input = r#"Traceback (most recent call last):
  File "main.py", line 5, in <module>
    foo()
  File "main.py", line 10, in foo
    raise ValueError("bad")

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
  File "main.py", line 5, in <module>
    foo()
  File "handler.py", line 20, in handle
    bar()"#;
        let frames = parse_python_traceback(input);
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0].file, "main.py");
        assert_eq!(frames[0].line, 5);
        assert_eq!(frames[3].file, "handler.py");
        assert_eq!(frames[3].line, 20);
    }

    #[test]
    fn test_parse_no_frames() {
        let input = "No traceback here";
        let frames = parse_python_traceback(input);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_parse_windows_paths() {
        let input = r#"  File "C:\Users\test\project\main.py", line 15, in run"#;
        let frames = parse_python_traceback(input);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].file, r"C:\Users\test\project\main.py");
        assert_eq!(frames[0].line, 15);
    }
}
