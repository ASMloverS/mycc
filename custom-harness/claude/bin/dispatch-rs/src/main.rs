mod frontmatter;
mod payload;
mod prompt;
mod registry;

use std::collections::HashMap;
use std::path::PathBuf;

fn harness_dir() -> PathBuf {
    std::env::current_exe()
        .expect("current_exe")
        .parent().unwrap()  // bin/
        .parent().unwrap()  // custom-harness/
        .to_path_buf()
}

fn die(msg: &str, code: i32) -> ! {
    eprintln!("dispatch: {msg}");
    std::process::exit(code);
}

fn err_code(e: &anyhow::Error) -> i32 {
    let m = e.to_string();
    if m.contains("Not found:") || m.contains("Unknown name:") || m.contains("Ambiguous name") { 2 }
    else if m.contains("MD not found:") || m.contains("Empty MD body:") { 3 }
    else { 1 }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let harness = harness_dir();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        let reg = registry::load_registry(&harness)
            .unwrap_or_else(|e| die(&e.to_string(), 4));
        let idx = args.iter().position(|a| a == "--help" || a == "-h").unwrap();
        let help_name = args.get(idx + 1).filter(|a| !a.starts_with('-')).map(|s| s.as_str());
        payload::print_help(&reg, &harness, help_name);
        return;
    }

    let mut model: Option<String> = None;
    let mut bg = false;
    let mut parallel = false;
    let mut inline = false;
    let mut clean: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" if i + 1 < args.len() => { model = Some(args[i + 1].clone()); i += 2; }
            "--bg"       => { bg = true;       i += 1; }
            "--parallel" => { parallel = true;  i += 1; }
            "--inline"   => { inline = true;    i += 1; }
            _            => { clean.push(args[i].clone()); i += 1; }
        }
    }

    let reg = registry::load_registry(&harness)
        .unwrap_or_else(|e| die(&e.to_string(), 4));
    let mut md_cache: HashMap<String, (HashMap<String, String>, String)> = HashMap::new();

    if parallel {
        if clean.is_empty() { payload::print_help(&reg, &harness, None); return; }
        let mut payloads = Vec::new();
        for token in &clean {
            let (name_tok, rest) = token.split_once(char::is_whitespace)
                .unwrap_or_else(|| die(&format!("--parallel token must be 'name prompt': got {token:?}"), 2));
            let p = payload::build_payload(&reg, &harness, name_tok, rest.trim_start(),
                model.as_deref(), bg, inline, &mut md_cache)
                .unwrap_or_else(|e| die(&e.to_string(), err_code(&e)));
            payloads.push(p);
        }
        let out = payload::Output { mode: "parallel", payloads };
        println!("{}", serde_json::to_string(&out).unwrap());
        return;
    }

    if clean.is_empty() { payload::print_help(&reg, &harness, None); return; }

    // Fallback: LLM may bundle "<name> <prompt>" into one quoted argv → split at first whitespace.
    let (name_tok, user_prompt) = if clean.len() == 1 {
        match clean[0].split_once(char::is_whitespace) {
            Some((n, p)) => (n.to_string(), p.trim_start().to_string()),
            None => (clean[0].clone(), String::new()),
        }
    } else {
        (clean[0].clone(), clean[1..].join(" "))
    };
    let p = payload::build_payload(&reg, &harness, &name_tok, &user_prompt,
        model.as_deref(), bg, inline, &mut md_cache)
        .unwrap_or_else(|e| die(&e.to_string(), err_code(&e)));
    let out = payload::Output { mode: "single", payloads: vec![p] };
    println!("{}", serde_json::to_string(&out).unwrap());
}
