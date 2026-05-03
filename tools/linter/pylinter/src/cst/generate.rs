use super::CSTSource;

impl CSTSource {
    pub fn regenerate(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                let mut buf = String::new();
                buf.push_str(&line.indent.raw);
                buf.push_str(&line.code);
                buf.push_str(&line.trailing_ws);
                if let Some(ref comment) = line.comment {
                    buf.push_str(comment);
                }
                if line.raw_content.ends_with("\r\n") {
                    buf.push_str("\r\n");
                } else if line.raw_content.ends_with('\n') {
                    buf.push('\n');
                } else if line.raw_content.ends_with('\r') {
                    buf.push('\r');
                }
                buf
            })
            .collect()
    }
}
