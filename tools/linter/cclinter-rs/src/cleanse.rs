use regex::Regex;
use std::sync::LazyLock;

static RE_C_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").unwrap());

pub struct CleansedLines {
    raw_lines: Vec<String>,
    lines_without_raw_strings: Vec<String>,
    lines: Vec<String>,
    elided: Vec<String>,
}

impl CleansedLines {
    pub fn from_source(source: &str) -> Self {
        let mut raw_lines = Vec::new();
        raw_lines.push(
            "// marker so line numbers and indices both start at 1".to_string(),
        );
        for line in source.lines() {
            raw_lines.push(line.to_string());
        }
        raw_lines.push("// marker so line numbers end in a known way".to_string());

        let text = raw_lines.join("\n");

        let after_mlc = remove_block_comments(&text);

        let after_raw = cleanse_raw_strings(&after_mlc);
        let lines_without_raw_strings: Vec<String> =
            after_raw.lines().map(|s| s.to_string()).collect();

        let lines: Vec<String> = after_raw.lines().map(cleanse_line).collect();

        let elided: Vec<String> = lines.iter().map(|l| collapse_strings(l)).collect();

        CleansedLines {
            raw_lines,
            lines_without_raw_strings,
            lines,
            elided,
        }
    }

    pub fn raw_lines(&self) -> &[String] {
        &self.raw_lines
    }

    pub fn lines_without_raw_strings(&self) -> &[String] {
        &self.lines_without_raw_strings
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn elided(&self) -> &[String] {
        &self.elided
    }

    pub fn len(&self) -> usize {
        self.raw_lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw_lines.is_empty()
    }
}

fn remove_block_comments(text: &str) -> String {
    RE_C_COMMENT
        .replace_all(text, |caps: &regex::Captures<'_>| {
            caps.get(0)
                .unwrap()
                .as_str()
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}

fn cleanse_raw_strings(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if chars[i] == 'R' && i + 1 < len && chars[i + 1] == '"' {
            let delim_start = i + 2;
            let mut paren_pos = delim_start;
            while paren_pos < len && chars[paren_pos] != '(' {
                paren_pos += 1;
            }
            if paren_pos >= len {
                result.push(chars[i]);
                i += 1;
                continue;
            }

            let delimiter: String = chars[delim_start..paren_pos].iter().collect();
            let close = format!("){}\"", delimiter);
            let close_chars: Vec<char> = close.chars().collect();

            let mut j = paren_pos + 1;
            let mut found_end = None;
            while j + close_chars.len() <= len {
                let mut ok = true;
                for (k, &cc) in close_chars.iter().enumerate() {
                    if chars[j + k] != cc {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    found_end = Some(j + close_chars.len());
                    break;
                }
                j += 1;
            }

            if let Some(end) = found_end {
                let nl = chars[i..end].iter().filter(|&&c| c == '\n').count();
                result.push('"');
                result.push('"');
                for _ in 0..nl {
                    result.push('\n');
                }
                i = end;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn cleanse_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;
    let mut in_single = false;
    let mut in_double = false;

    while i < len {
        if !in_single && !in_double {
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
                break;
            }
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                let mut j = i + 2;
                while j + 1 < len && !(chars[j] == '*' && chars[j + 1] == '/') {
                    j += 1;
                }
                if j + 1 < len && chars[j] == '*' && chars[j + 1] == '/' {
                    i = j + 2;
                    continue;
                }
                break;
            }
            if chars[i] == '"' {
                in_double = true;
            } else if chars[i] == '\'' {
                in_single = true;
            }
            result.push(chars[i]);
        } else if in_double {
            if chars[i] == '\\' && i + 1 < len {
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == '"' {
                in_double = false;
            }
            result.push(chars[i]);
        } else {
            if chars[i] == '\\' && i + 1 < len {
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == '\'' {
                in_single = false;
            }
            result.push(chars[i]);
        }
        i += 1;
    }

    result
}

fn collapse_strings(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if chars[i] == '"' {
            result.push('"');
            i += 1;
            while i < len {
                if chars[i] == '\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    result.push('"');
                    i += 1;
                    break;
                }
                i += 1;
            }
        } else if chars[i] == '\'' {
            let is_digit_sep = i > 0
                && i + 1 < len
                && chars[i - 1].is_ascii_hexdigit()
                && chars[i + 1].is_ascii_hexdigit();
            if is_digit_sep {
                result.push('\'');
                i += 1;
            } else {
                result.push('\'');
                i += 1;
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '\'' {
                        result.push('\'');
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_arrays_equal_length() {
        let cl = CleansedLines::from_source("int x = 1;\nint y = 2;");
        assert_eq!(cl.raw_lines.len(), cl.lines_without_raw_strings.len());
        assert_eq!(cl.raw_lines.len(), cl.lines.len());
        assert_eq!(cl.raw_lines.len(), cl.elided.len());
    }

    #[test]
    fn test_comments_stripped() {
        let cl = CleansedLines::from_source("int x = 1; // comment");
        assert!(!cl.lines[1].contains("comment"));
        assert!(cl.lines[1].contains("int x = 1;"));
    }

    #[test]
    fn test_block_comment_removed() {
        let cl = CleansedLines::from_source("int x = /* block */ 1;");
        assert!(!cl.lines[1].contains("block"));
        assert!(cl.lines[1].contains("int x ="));
        assert!(cl.lines[1].contains("1;"));
    }

    #[test]
    fn test_multiline_block_comment() {
        let source = "int x = /* start\nmiddle\nend */ 1;";
        let cl = CleansedLines::from_source(source);
        assert!(!cl.lines[1].contains("start"));
        assert!(!cl.lines[2].contains("middle"));
        assert!(!cl.lines[3].contains("end"));
        assert!(cl.lines[3].contains("1;"));
        assert_eq!(cl.len(), 5);
    }

    #[test]
    fn test_strings_collapsed_in_elided() {
        let cl = CleansedLines::from_source(r#"int x = "hello world";"#);
        assert!(cl.elided[1].contains("\"\""));
        assert!(!cl.elided[1].contains("hello world"));
    }

    #[test]
    fn test_raw_string_replaced() {
        let cl = CleansedLines::from_source(r#"auto s = R"(hello)";"#);
        assert!(!cl.lines_without_raw_strings[1].contains("hello"));
    }

    #[test]
    fn test_marker_lines() {
        let cl = CleansedLines::from_source("int x = 1;");
        assert_eq!(cl.len(), 3);
        assert!(cl.raw_lines[0].contains("marker so line numbers and indices both start at 1"));
        assert!(cl.raw_lines[2].contains("marker so line numbers end in a known way"));
    }

    #[test]
    fn test_digit_separator_preserved() {
        let cl = CleansedLines::from_source("int x = 1'000;");
        assert!(cl.elided[1].contains("1'000"));
    }
}
