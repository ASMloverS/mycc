pub fn find_closing_quote(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return i;
        }
        i += 1;
    }
    line.len()
}

pub fn count_format_specs(fmt: &str) -> usize {
    let bytes = fmt.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
                i += 2;
                continue;
            }
            count += 1;
        }
        i += 1;
    }
    count
}

pub fn count_args(s: &str) -> usize {
    let s = s.trim();
    if s.is_empty() || s.starts_with(')') {
        return 0;
    }
    let mut depth = 0i32;
    let mut count = 1;
    for c in s.chars() {
        match c {
            '(' | '<' => depth += 1,
            ')' | '>' => {
                depth -= 1;
                if depth < 0 {
                    break;
                }
            }
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

pub fn contains_outside_string(line: &str, needle: &str) -> bool {
    let bytes = line.as_bytes();
    let needle_bytes = needle.as_bytes();
    let needle_len = needle_bytes.len();
    let mut quote_char: Option<u8> = None;
    let mut i = 0;

    while i < bytes.len() {
        match quote_char {
            None => {
                if bytes[i] == b'"' || bytes[i] == b'\'' {
                    quote_char = Some(bytes[i]);
                    i += 1;
                    continue;
                }
                if i + needle_len <= bytes.len() && &bytes[i..i + needle_len] == needle_bytes {
                    return true;
                }
            }
            Some(qc) => {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == qc {
                    quote_char = None;
                }
            }
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_outside_string_real_comment() {
        assert!(contains_outside_string("int x; /* comment */", "/*"));
    }

    #[test]
    fn test_contains_outside_string_in_string() {
        assert!(!contains_outside_string(r#"s = "/* */";"#, "/*"));
    }

    #[test]
    fn test_contains_outside_string_mixed() {
        assert!(contains_outside_string("/* comment */ code", "/*"));
    }
}
