#!/usr/bin/env python3
"""dispatch.py — read registry + md → print JSON for Claude to spawn subagent."""

import io
import json
import re
import sys
from pathlib import Path

HARNESS_DIR = Path(__file__).resolve().parents[1]
REGISTRY_PATH = HARNESS_DIR / "registry.yaml"


# ---------------------------------------------------------------------------
# Minimal YAML parser — supports only the registry.yaml structure
# ---------------------------------------------------------------------------

def _parse_registry_yaml(text: str) -> dict:
    try:
        import yaml
        return yaml.safe_load(text)
    except ImportError:
        pass

    result: dict = {}
    current_section: str | None = None
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        m_hdr = re.match(r"^([a-z_]+):\s*(?:\{\s*\})?$", stripped)
        if m_hdr:
            current_section = m_hdr.group(1)
            result[current_section] = {}
            continue
        m = re.match(
            r"^\s{2}([\w-]+):\s*\{[^}]*path:\s*([^\s,}]+)[^}]*desc:\s*\"([^\"]+)\"",
            line,
        )
        if m and current_section:
            name, path, desc = m.group(1), m.group(2), m.group(3)
            result[current_section][name] = {"path": path, "desc": desc}
    return result


# ---------------------------------------------------------------------------
# Registry helpers
# ---------------------------------------------------------------------------

def load_registry() -> dict:
    if not REGISTRY_PATH.exists():
        _die(f"registry.yaml not found: {REGISTRY_PATH}", 4)
    return _parse_registry_yaml(REGISTRY_PATH.read_text(encoding="utf-8"))


def resolve_name(registry: dict, token: str) -> tuple[str, str, dict]:
    """Return (type, name, entry). Exits on ambiguity or unknown name."""
    if ":" in token:
        type_, name = token.split(":", 1)
        if name:  # type:name format
            section = registry.get(type_)
            if section and name in section:
                return type_, name, section[name]
            _die(f"Not found: {token!r}", 2)
        else:
            token = type_  # trailing colon (e.g. "dev-cycle:") → strip, fall through

    matches = []
    for type_, section in registry.items():
        if token in section:
            matches.append((type_, token, section[token]))

    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        opts = " / ".join(f"{t}:{token}" for t, _, _ in matches)
        _die(f"Ambiguous name {token!r}. Use: {opts}", 2)
    _die(f"Unknown name: {token!r}. Run --help to list registered items.", 2)


# ---------------------------------------------------------------------------
# Frontmatter + prompt assembly
# ---------------------------------------------------------------------------

_FM_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n", re.DOTALL)


def parse_md(md_path: Path) -> tuple[dict, str]:
    text = md_path.read_text(encoding="utf-8")
    m = _FM_RE.match(text)
    if not m:
        return {}, text
    fm_raw = m.group(1)
    body = text[m.end():]
    fm: dict = {}
    for line in fm_raw.splitlines():
        if ":" in line:
            k, _, v = line.partition(":")
            fm[k.strip()] = v.strip().strip('"')
    return fm, body


def assemble_prompt(type_: str, name: str, body: str, fm: dict, user_prompt: str,
                    harness_dir: Path | None = None) -> str:
    tools = fm.get("tools", "")
    tool_hint = (
        f"\nTool access guidance (soft): originally authored for tools = [{tools}]."
        f"\nPrefer those; avoid unrelated tools.\n"
        if tools
        else ""
    )
    harness_hint = f"\nHARNESS_DIR = {harness_dir}\n" if harness_dir else ""
    return (
        "You are a one-shot general-purpose subagent executing the definition below.\n"
        "Follow it literally.\n\n"
        f"<!-- BEGIN: {type_}/{name} -->\n"
        f"{body.strip()}\n"
        "<!-- END -->\n"
        f"{tool_hint}"
        f"{harness_hint}"
        "\n---\n## User Input\n"
        f"{user_prompt}"
    )


def assemble_thin_prompt(type_: str, name: str, fm: dict, user_prompt: str,
                         md_path: Path, harness_dir: Path | None = None) -> str:
    """Thin-pointer prompt: subagent reads the definition file itself."""
    tools = fm.get("tools", "")
    tool_hint = f"TOOL_HINT: {tools}" if tools else "TOOL_HINT:"
    harness_line = f"HARNESS_DIR: {harness_dir}" if harness_dir else ""
    return (
        "You are a one-shot subagent. Read the file at DEFINITION_FILE as your\n"
        "literal instructions, then process the User Input.\n\n"
        f"DEFINITION_FILE: {md_path}\n"
        f"{harness_line}\n"
        f"TYPE: {type_}/{name}\n"
        f"{tool_hint}\n"
        "\n---\n## User Input\n"
        f"{user_prompt}"
    )


# ---------------------------------------------------------------------------
# --help output
# ---------------------------------------------------------------------------

def print_help(registry: dict, name: str | None = None) -> None:
    if name:
        for type_, section in registry.items():
            if name in section:
                e = section[name]
                md_path = HARNESS_DIR / e["path"]
                fm, _ = parse_md(md_path) if md_path.exists() else ({}, "")
                print(f"[{type_}] {name}")
                print(f"  desc: {e['desc']}")
                for k, v in fm.items():
                    print(f"  {k}: {v}")
                return
        _die(f"Unknown name: {name!r}", 2)

    print("Usage: dispatch <name|type:name> <prompt> [--model M] [--bg] [--help [name]]\n")
    for type_, section in registry.items():
        print(f"[{type_}]")
        for n, e in section.items():
            print(f"  {n:<20} {e['desc']}")
        print()


# ---------------------------------------------------------------------------
# Utility
# ---------------------------------------------------------------------------

def _die(msg: str, code: int = 1) -> None:
    print(f"dispatch: {msg}", file=sys.stderr)
    sys.exit(code)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def build_payload(registry: dict, name_token: str, user_prompt: str,
                  model: str | None = None, bg: bool = False,
                  inline: bool = False, _md_cache: dict | None = None) -> dict:
    type_, name, entry = resolve_name(registry, name_token)
    md_path = HARNESS_DIR / entry["path"]
    if not md_path.exists():
        _die(f"MD not found: {md_path}", 3)

    path_key = entry["path"]
    if _md_cache is not None and path_key in _md_cache:
        fm, body = _md_cache[path_key]
    else:
        fm, body = parse_md(md_path)
        if _md_cache is not None:
            _md_cache[path_key] = (fm, body)

    if not body.strip():
        _die(f"Empty MD body: {md_path}", 3)

    if inline:
        prompt = assemble_prompt(type_, name, body, fm, user_prompt, harness_dir=HARNESS_DIR)
    else:
        prompt = assemble_thin_prompt(type_, name, fm, user_prompt, md_path, harness_dir=HARNESS_DIR)

    payload: dict = {
        "subagent_type": "general-purpose",
        "description": entry["desc"][:50],
        "prompt": prompt,
    }
    effective_model = model or fm.get("model")
    if effective_model:
        payload["model"] = effective_model
    if bg:
        payload["run_in_background"] = True
    return payload


def main() -> None:
    args = sys.argv[1:]

    # --help [name]
    if "--help" in args or "-h" in args:
        registry = load_registry()
        idx = args.index("--help") if "--help" in args else args.index("-h")
        help_name = args[idx + 1] if idx + 1 < len(args) and not args[idx + 1].startswith("-") else None
        print_help(registry, help_name)
        return

    # Parse flags
    model: str | None = None
    bg = False
    parallel = False
    inline = False
    clean: list[str] = []
    i = 0
    while i < len(args):
        if args[i] == "--model" and i + 1 < len(args):
            model = args[i + 1]
            i += 2
        elif args[i] == "--bg":
            bg = True
            i += 1
        elif args[i] == "--parallel":
            parallel = True
            i += 1
        elif args[i] == "--inline":
            inline = True
            i += 1
        else:
            clean.append(args[i])
            i += 1

    registry = load_registry()

    # --parallel: each remaining arg is "name prompt…" (first word = name, rest = prompt)
    if parallel:
        if not clean:
            print_help(registry)
            sys.exit(0)
        payloads = []
        md_cache: dict = {}
        for token in clean:
            parts = token.split(None, 1)
            if len(parts) < 2:
                _die(f"--parallel token must be 'name prompt': got {token!r}", 2)
            payloads.append(build_payload(registry, parts[0], parts[1], model, bg, inline, md_cache))
        print(json.dumps({"mode": "parallel", "payloads": payloads}, ensure_ascii=False, separators=(",", ":")))
        return

    if not clean:
        print_help(registry)
        sys.exit(0)

    name_token = clean[0]
    user_prompt = " ".join(clean[1:])
    print(json.dumps(
        {"mode": "single", "payloads": [build_payload(registry, name_token, user_prompt, model, bg, inline)]},
        ensure_ascii=False, separators=(",", ":"),
    ))


if __name__ == "__main__":
    # Ensure UTF-8 output on Windows when running as CLI
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")
    main()
