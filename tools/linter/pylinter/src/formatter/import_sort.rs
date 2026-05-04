use crate::config::{FormatConfig, ImportSorting};
use crate::cst::{CSTLine, CSTSource, IndentInfo};

pub fn fix_import_sort(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.import_sorting == ImportSorting::Disabled {
        return Ok(());
    }
    if source.lines.is_empty() {
        return Ok(());
    }

    let blocks = find_import_blocks(&source.lines);
    if blocks.is_empty() {
        return Ok(());
    }

    let mut new_lines = Vec::new();
    let mut cursor = 0usize;

    for (start, end) in blocks {
        if cursor < start {
            new_lines.extend(source.lines[cursor..start].to_vec());
        }

        let block_lines = &source.lines[start..end];
        let sorted = sort_block(block_lines);
        new_lines.extend(sorted);

        cursor = end;
    }

    if cursor < source.lines.len() {
        new_lines.extend(source.lines[cursor..].to_vec());
    }

    source.lines = new_lines;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ImportGroup {
    Stdlib,
    ThirdParty,
    Local,
}

#[derive(Clone, Debug)]
struct ImportEntry {
    module: String,
    names: Option<Vec<String>>,
    as_names: Vec<Option<String>>,
    group: ImportGroup,
    comment: Option<String>,
    leading_comment: Option<String>,
}

fn zero_indent() -> IndentInfo {
    IndentInfo {
        level: 0,
        raw: String::new(),
        width: 0,
        uses_tabs: false,
    }
}

fn is_import_line(line: &CSTLine) -> bool {
    !line.is_blank
        && line.indent.level == 0
        && (line.code.starts_with("import ") || line.code.starts_with("from "))
}

fn count_parens(code: &str) -> i32 {
    code.chars().fold(0i32, |d, c| match c {
        '(' => d + 1,
        ')' => d - 1,
        _ => d,
    })
}

fn find_import_blocks(lines: &[CSTLine]) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if is_import_line(&lines[i]) {
            let start = i;
            let mut end = i;
            let mut paren_depth = count_parens(&lines[i].code);

            loop {
                if end >= lines.len() {
                    break;
                }

                if paren_depth > 0 {
                    end += 1;
                    if end < lines.len() {
                        paren_depth += count_parens(&lines[end].code);
                    }
                    if paren_depth <= 0 {
                        end += 1;
                    }
                    continue;
                }

                if is_import_line(&lines[end]) {
                    paren_depth = count_parens(&lines[end].code);
                    end += 1;
                } else if lines[end].is_blank
                    && end + 1 < lines.len()
                    && is_import_line(&lines[end + 1])
                {
                    end += 1;
                } else if lines[end].comment.is_some()
                    && lines[end].code.is_empty()
                    && lines[end].indent.level == 0
                {
                    end += 1;
                } else {
                    break;
                }
            }

            blocks.push((start, end));
            i = end;
        } else {
            i += 1;
        }
    }

    blocks
}

fn classify_import(module: &str) -> ImportGroup {
    if module.starts_with('.') {
        return ImportGroup::Local;
    }
    let first_segment = module.split('.').next().unwrap_or(module);
    if STDLIB_MODULES.binary_search(&first_segment).is_ok() {
        ImportGroup::Stdlib
    } else {
        ImportGroup::ThirdParty
    }
}

fn parse_import_entries(code: &str) -> Vec<ImportEntry> {
    let code = code.trim_end();

    if let Some(rest) = code.strip_prefix("from ") {
        let parts: Vec<&str> = rest.splitn(2, " import ").collect();
        if parts.len() != 2 {
            return Vec::new();
        }
        let module = parts[0].trim().to_string();
        let names_str = parts[1].trim();

        let (names, as_names) = parse_import_names(names_str);
        let group = classify_import(&module);

        return vec![ImportEntry {
            module,
            names: Some(names),
            as_names,
            group,
            comment: None,
            leading_comment: None,
        }];
    }

    if let Some(rest) = code.strip_prefix("import ") {
        let rest = rest.trim();
        let parts: Vec<&str> = rest.split(',').map(|p| p.trim()).collect();
        let mut entries = Vec::new();

        for part in parts {
            if part.is_empty() {
                continue;
            }
            let mut module = part.to_string();
            let mut as_name: Option<String> = None;
            if let Some(pos) = module.find(" as ") {
                as_name = Some(module[pos + 4..].trim().to_string());
                module = module[..pos].to_string();
            }

            let group = classify_import(&module);

            entries.push(ImportEntry {
                module,
                names: None,
                as_names: vec![as_name],
                group,
                comment: None,
                leading_comment: None,
            });
        }

        return entries;
    }

    Vec::new()
}

fn parse_import_names(names_str: &str) -> (Vec<String>, Vec<Option<String>>) {
    let mut names = Vec::new();
    let mut as_names = Vec::new();

    let inner = names_str
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(names_str);

    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(pos) = part.find(" as ") {
            names.push(part[..pos].trim().to_string());
            as_names.push(Some(part[pos + 4..].trim().to_string()));
        } else {
            names.push(part.trim().to_string());
            as_names.push(None);
        }
    }

    (names, as_names)
}

fn format_entry(entry: &ImportEntry) -> String {
    match &entry.names {
        Some(names) => {
            let parts: Vec<String> = names
                .iter()
                .enumerate()
                .map(|(i, name)| match entry.as_names.get(i).and_then(|a| a.as_ref()) {
                    Some(as_name) => format!("{} as {}", name, as_name),
                    None => name.clone(),
                })
                .collect();
            format!("from {} import {}", entry.module, parts.join(", "))
        }
        None => match entry.as_names.first().and_then(|a| a.as_ref()) {
            Some(as_name) => format!("import {} as {}", entry.module, as_name),
            None => format!("import {}", entry.module),
        },
    }
}

fn join_multiline_code(block_lines: &[CSTLine], start_idx: usize) -> (String, usize) {
    let first_code = &block_lines[start_idx].code;
    let depth = count_parens(first_code);
    if depth <= 0 {
        return (first_code.clone(), start_idx);
    }

    let mut combined = String::from(first_code);
    let mut current_depth = depth;
    let mut end_idx = start_idx;

    for j in (start_idx + 1)..block_lines.len() {
        end_idx = j;
        let cont_code = block_lines[j].code.trim();
        if cont_code.is_empty() {
            continue;
        }
        combined.push(' ');
        combined.push_str(cont_code);
        current_depth += count_parens(cont_code);
        if current_depth <= 0 {
            break;
        }
    }

    (combined, end_idx)
}

fn sort_block(block_lines: &[CSTLine]) -> Vec<CSTLine> {
    let mut entries: Vec<ImportEntry> = Vec::new();
    let mut pending_comment: Option<String> = None;
    let mut i = 0;

    while i < block_lines.len() {
        let line = &block_lines[i];

        if line.is_blank {
            i += 1;
            continue;
        }

        if line.code.is_empty() && line.comment.is_some() {
            pending_comment = line.comment.clone();
            i += 1;
            continue;
        }

        if is_import_line(line) {
            let (logical_code, end_idx) = join_multiline_code(block_lines, i);
            let parsed = parse_import_entries(&logical_code);

            for (idx, mut entry) in parsed.into_iter().enumerate() {
                if let Some(comment) = pending_comment.take() {
                    entry.leading_comment = Some(comment);
                } else if idx == 0 && line.comment.is_some() {
                    entry.comment = line.comment.clone();
                }
                entries.push(entry);
            }

            i = end_idx + 1;
            continue;
        }

        i += 1;
    }

    if entries.is_empty() {
        return block_lines.to_vec();
    }

    for entry in &mut entries {
        if let Some(ref mut names) = entry.names {
            sort_names_with_as(names, &mut entry.as_names);
        }
    }

    merge_from_imports(&mut entries);
    entries.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then_with(|| a.module.cmp(&b.module))
            .then_with(|| a.names.is_some().cmp(&b.names.is_some()))
    });

    let mut result = Vec::new();
    let mut prev_group: Option<&ImportGroup> = None;

    for entry in &entries {
        if prev_group.is_some() && prev_group != Some(&entry.group) {
            result.push(make_blank_line());
        }
        prev_group = Some(&entry.group);

        if let Some(ref lc) = entry.leading_comment {
            result.push(CSTLine {
                num: 0,
                indent: zero_indent(),
                tokens: Vec::new(),
                raw_content: format!("{}\n", lc),
                code: String::new(),
                trailing_ws: String::new(),
                comment: Some(lc.clone()),
                is_blank: false,
            });
        }

        let code = format_entry(entry);
        let trailing_ws = if entry.comment.is_some() {
            "  ".to_string()
        } else {
            String::new()
        };

        result.push(CSTLine {
            num: 0,
            indent: zero_indent(),
            tokens: Vec::new(),
            raw_content: format!("{}\n", code),
            code,
            trailing_ws,
            comment: entry.comment.clone(),
            is_blank: false,
        });
    }

    result
}

fn merge_from_imports(entries: &mut Vec<ImportEntry>) {
    let mut merged: Vec<ImportEntry> = Vec::new();

    for entry in entries.drain(..) {
        if let Some(idx) = merged.iter().position(|e| {
            e.module == entry.module
                && e.names.is_some()
                && entry.names.is_some()
                && e.group == entry.group
        }) {
            if let (Some(ref new_names), ref as_names) = (&entry.names, &entry.as_names) {
                let existing = &mut merged[idx];
                if let Some(ref mut existing_names) = existing.names {
                    for (i, name) in new_names.iter().enumerate() {
                        if !existing_names.contains(name) {
                            existing_names.push(name.clone());
                            existing.as_names.push(as_names.get(i).and_then(|a| a.clone()));
                        }
                    }
                    sort_names_with_as(existing_names, &mut existing.as_names);
                }
            }
            if entry.comment.is_some() && merged[idx].comment.is_none() {
                merged[idx].comment = entry.comment;
            }
            if entry.leading_comment.is_some() && merged[idx].leading_comment.is_none() {
                merged[idx].leading_comment = entry.leading_comment;
            }
        } else {
            merged.push(entry);
        }
    }

    *entries = merged;
}

fn sort_names_with_as(names: &mut [String], as_names: &mut [Option<String>]) {
    let mut pairs: Vec<(String, Option<String>)> = names
        .iter()
        .zip(as_names.iter())
        .map(|(n, a)| (n.clone(), a.clone()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    for (i, (name, as_name)) in pairs.into_iter().enumerate() {
        names[i] = name;
        as_names[i] = as_name;
    }
}

fn make_blank_line() -> CSTLine {
    CSTLine {
        num: 0,
        indent: zero_indent(),
        tokens: Vec::new(),
        raw_content: "\n".to_string(),
        code: String::new(),
        trailing_ws: String::new(),
        comment: None,
        is_blank: true,
    }
}

const STDLIB_MODULES: &[&str] = &[
    "__future__",
    "_thread",
    "abc",
    "aifc",
    "argparse",
    "array",
    "ast",
    "asynchat",
    "asyncio",
    "asyncore",
    "atexit",
    "audioop",
    "base64",
    "bdb",
    "binascii",
    "binhex",
    "bisect",
    "builtins",
    "bz2",
    "calendar",
    "cgi",
    "cgitb",
    "chunk",
    "cmath",
    "cmd",
    "code",
    "codecs",
    "codeop",
    "collections",
    "colorsys",
    "compileall",
    "concurrent",
    "configparser",
    "contextlib",
    "contextvars",
    "copy",
    "copyreg",
    "cProfile",
    "crypt",
    "csv",
    "ctypes",
    "curses",
    "dataclasses",
    "datetime",
    "dbm",
    "decimal",
    "difflib",
    "dis",
    "distutils",
    "doctest",
    "email",
    "encodings",
    "enum",
    "errno",
    "faulthandler",
    "fcntl",
    "filecmp",
    "fileinput",
    "fnmatch",
    "fractions",
    "ftplib",
    "functools",
    "gc",
    "getopt",
    "getpass",
    "gettext",
    "glob",
    "graphlib",
    "grp",
    "gzip",
    "hashlib",
    "heapq",
    "hmac",
    "html",
    "http",
    "idlelib",
    "imaplib",
    "imghdr",
    "imp",
    "importlib",
    "inspect",
    "io",
    "ipaddress",
    "itertools",
    "json",
    "keyword",
    "lib2to3",
    "linecache",
    "locale",
    "logging",
    "lzma",
    "mailbox",
    "mailcap",
    "marshal",
    "math",
    "mimetypes",
    "mmap",
    "modulefinder",
    "multiprocessing",
    "netrc",
    "nis",
    "nntplib",
    "numbers",
    "operator",
    "optparse",
    "os",
    "ossaudiodev",
    "pathlib",
    "pdb",
    "pickle",
    "pickletools",
    "pipes",
    "pkgutil",
    "platform",
    "plistlib",
    "poplib",
    "posix",
    "posixpath",
    "pprint",
    "profile",
    "pstats",
    "pty",
    "pwd",
    "py_compile",
    "pyclbr",
    "pydoc",
    "queue",
    "quopri",
    "random",
    "re",
    "readline",
    "reprlib",
    "resource",
    "rlcompleter",
    "runpy",
    "sched",
    "secrets",
    "select",
    "selectors",
    "shelve",
    "shlex",
    "shutil",
    "signal",
    "site",
    "smtpd",
    "smtplib",
    "sndhdr",
    "socket",
    "socketserver",
    "spwd",
    "sqlite3",
    "ssl",
    "stat",
    "statistics",
    "string",
    "stringprep",
    "struct",
    "subprocess",
    "sunau",
    "symtable",
    "sys",
    "sysconfig",
    "syslog",
    "tabnanny",
    "tarfile",
    "telnetlib",
    "tempfile",
    "termios",
    "test",
    "textwrap",
    "threading",
    "time",
    "timeit",
    "tkinter",
    "token",
    "tokenize",
    "tomllib",
    "trace",
    "traceback",
    "tracemalloc",
    "tty",
    "turtle",
    "turtledemo",
    "types",
    "typing",
    "unicodedata",
    "unittest",
    "urllib",
    "uu",
    "uuid",
    "venv",
    "warnings",
    "wave",
    "weakref",
    "webbrowser",
    "winreg",
    "winsound",
    "wsgiref",
    "xdrlib",
    "xml",
    "xmlrpc",
    "zipapp",
    "zipfile",
    "zipimport",
    "zlib",
    "zoneinfo",
];

#[cfg(test)]
mod tests {
    use crate::config::{FormatConfig, ImportSorting};
    use crate::cst::CSTSource;

    fn sorted(input: &str) -> String {
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_import_sort(&mut cst, &FormatConfig::default()).unwrap();
        cst.regenerate()
    }

    fn sorted_disabled(input: &str) -> String {
        let mut config = FormatConfig::default();
        config.import_sorting = ImportSorting::Disabled;
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_import_sort(&mut cst, &config).unwrap();
        cst.regenerate()
    }

    #[test]
    fn sort_stdlib_imports() {
        assert_eq!(sorted("import sys\nimport os\n"), "import os\nimport sys\n");
    }

    #[test]
    fn group_stdlib_and_third_party() {
        assert_eq!(
            sorted("import requests\nimport os\nimport sys\n"),
            "import os\nimport sys\n\nimport requests\n"
        );
    }

    #[test]
    fn sort_from_imports() {
        assert_eq!(
            sorted("from os import path\nfrom os import environ\n"),
            "from os import environ, path\n"
        );
    }

    #[test]
    fn no_imports_no_change() {
        let input = "x = 1\n";
        assert_eq!(sorted(input), input);
    }

    #[test]
    fn preserve_indent_in_type_checking_block() {
        let input = "from typing import TYPE_CHECKING\n\nif TYPE_CHECKING:\n    import os\n    import sys\n";
        assert_eq!(sorted(input), input);
    }

    #[test]
    fn three_groups_with_relative_import() {
        assert_eq!(
            sorted("import requests\nfrom .utils import helper\nimport os\n"),
            "import os\n\nimport requests\n\nfrom .utils import helper\n"
        );
    }

    #[test]
    fn import_with_as() {
        assert_eq!(
            sorted("import numpy as np\nimport os\n"),
            "import os\n\nimport numpy as np\n"
        );
    }

    #[test]
    fn from_import_with_as() {
        assert_eq!(
            sorted("from os import path as p\nfrom os import environ\n"),
            "from os import environ, path as p\n"
        );
    }

    #[test]
    fn disabled_config_no_change() {
        let input = "import sys\nimport os\n";
        assert_eq!(sorted_disabled(input), input);
    }

    #[test]
    fn code_after_imports_preserved() {
        assert_eq!(
            sorted("import sys\nimport os\n\ndef foo():\n    pass\n"),
            "import os\nimport sys\n\ndef foo():\n    pass\n"
        );
    }

    #[test]
    fn already_sorted_no_change() {
        let input = "import os\nimport sys\n\nimport requests\n";
        assert_eq!(sorted(input), input);
    }

    #[test]
    fn from_import_no_merge_different_module() {
        assert_eq!(
            sorted("from os import path\nfrom sys import argv\nimport os\n"),
            "import os\nfrom os import path\nfrom sys import argv\n"
        );
    }

    #[test]
    fn multiline_from_import_sorted() {
        assert_eq!(
            sorted("from os import (\n    path,\n    environ,\n)\n"),
            "from os import environ, path\n"
        );
    }

    #[test]
    fn comment_inside_import_block_preserved() {
        assert_eq!(
            sorted("import os\n# system\nimport sys\n"),
            "import os\n# system\nimport sys\n"
        );
    }

    #[test]
    fn inline_comment_whitespace_preserved() {
        assert_eq!(
            sorted("import os  # system\nimport sys\n"),
            "import os  # system\nimport sys\n"
        );
    }

    #[test]
    fn comma_separated_bare_imports() {
        assert_eq!(sorted("import sys, os\n"), "import os\nimport sys\n");
    }

    #[test]
    fn single_from_import_names_sorted() {
        assert_eq!(
            sorted("from os import path, environ\n"),
            "from os import environ, path\n"
        );
    }

    #[test]
    fn multiple_import_blocks() {
        assert_eq!(
            sorted("import sys\nimport os\n\nx = 1\n\nimport flask\nimport requests\n"),
            "import os\nimport sys\n\nx = 1\n\nimport flask\nimport requests\n"
        );
    }
}
