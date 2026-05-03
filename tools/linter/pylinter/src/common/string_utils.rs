pub fn split_outside_strings(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut string_delim = ' ';
    for ch in s.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            current.push(ch);
            escape_next = true;
            continue;
        }
        if in_string {
            current.push(ch);
            if ch == string_delim {
                in_string = false;
                parts.push(current.clone());
                current.clear();
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            in_string = true;
            string_delim = ch;
            current.push(ch);
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic() {
        let parts = split_outside_strings("hello world foo");
        assert_eq!(parts, vec!["hello world foo"]);
    }

    #[test]
    fn split_with_strings() {
        let parts = split_outside_strings("x = \"hello world\" + y");
        assert_eq!(parts, vec!["x = ", "\"hello world\"", " + y"]);
    }
}
