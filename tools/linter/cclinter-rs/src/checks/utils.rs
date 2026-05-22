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
