#!/usr/bin/env node

import { readFileSync, openSync, readSync, closeSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const HOME = homedir().replace(/\\/g, "/");
const MAGENTA = "\x1b[35m";
const CYAN = "\x1b[36m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const BLUE = "\x1b[34m";
const RED = "\x1b[31m";
const DIM = "\x1b[2m";
const BOLD = "\x1b[1m";
const RESET = "\x1b[0m";
const SEP = `${DIM}\u2502${RESET}`;

const shortenPath = (p) => {
  const norm = p.replace(/\\/g, "/");
  if (norm.startsWith(HOME + "/")) return "~/" + norm.slice(HOME.length + 1);
  if (norm === HOME) return "~";
  return norm;
};

const getGitBranch = (cwd) => {
  try {
    const head = readFileSync(join(cwd, ".git", "HEAD"), "utf8").trim();
    const m = head.match(/^ref: refs\/heads\/(.+)$/);
    return m ? m[1] : null;
  } catch {
    return null;
  }
};

const fmtPct = (v) => v.toFixed(1);

const progressBar = (pct, color) => {
  const filled = Math.round(pct / 10);
  return color + "\u2588".repeat(filled) + DIM + "\u2591".repeat(10 - filled) + RESET;
};

// Read last `size` bytes of file. Returns {text, truncated}.
const readFileTail = (filePath, size) => {
  const normPath = filePath.replace(/\\/g, "/");
  const stat = statSync(normPath);
  const readSize = Math.min(stat.size, size);
  if (readSize === 0) return { text: "", truncated: false };
  const fd = openSync(normPath, "r");
  try {
    const buf = Buffer.alloc(readSize);
    readSync(fd, buf, 0, readSize, stat.size - readSize);
    return { text: buf.toString("utf8"), truncated: stat.size > size };
  } finally {
    closeSync(fd);
  }
};

const getRunningSubagents = (transcriptPath) => {
  if (!transcriptPath) return [];
  try {
    const { text, truncated } = readFileTail(transcriptPath, 32 * 1024);
    const lines = text.split("\n");
    // If we read a partial tail, skip the first (potentially cut-off) line.
    const safeLines = (truncated && lines.length > 1) ? lines.slice(1) : lines;

    const pending = new Map();
    for (const line of safeLines) {
      if (!line.trim()) continue;
      let obj;
      try { obj = JSON.parse(line); } catch { continue; }

      if (obj.type === "assistant") {
        for (const c of (obj.message?.content ?? [])) {
          if (c.type === "tool_use" && c.name === "Agent") {
            const label = c.input?.subagent_type || c.input?.description?.slice(0, 24) || "agent";
            pending.set(c.id, label);
          }
        }
      } else if (obj.type === "user") {
        for (const c of (obj.message?.content ?? [])) {
          if (c?.type === "tool_result") pending.delete(c.tool_use_id);
        }
      }
    }
    return [...pending.values()];
  } catch {
    return [];
  }
};

const chunks = [];
process.stdin.on("data", (d) => chunks.push(d));
process.stdin.on("end", () => {
  let data;
  try { data = JSON.parse(Buffer.concat(chunks).toString()); } catch { process.exit(1); }

  const dir = shortenPath(data.cwd || "?");
  const model = data.model?.display_name || data.model?.id || "?";
  const branch = getGitBranch(data.cwd || "");

  const ctx = data.context_window || {};
  const rawPct = ctx.used_percentage ?? 0;
  const ctxColor = rawPct >= 80 ? RED : rawPct >= 50 ? YELLOW : BLUE;

  const cost = data.cost?.total_cost_usd ?? 0;
  const dot = cost > 0 ? `${GREEN}\u25CF${RESET} ` : `${DIM}\u25CB${RESET} `;

  // Running-as-agent (team/swarm mode): data.agent.name is set.
  const selfAgent = data.agent?.name || data.agent?.type || null;

  const subagents = getRunningSubagents(data.transcript_path);

  const parts = [`${BOLD}${MAGENTA}${dir}${RESET}`];
  if (branch) parts.push(`${BOLD}${YELLOW}${branch}${RESET}`);
  parts.push(`${BOLD}${GREEN}${model}${RESET}`);
  if (selfAgent) parts.push(`${BOLD}${MAGENTA}[${selfAgent}]${RESET}`);
  parts.push(`${ctxColor}ctx: ${progressBar(rawPct, ctxColor)} ${ctxColor}${fmtPct(rawPct)}%${RESET}`);

  let out = dot + parts.join(` ${SEP} `);
  if (subagents.length > 0) {
    const label = subagents.map(s => `${CYAN}\u25B6 ${BOLD}${s}${RESET}`).join(`  `);
    out += `\n  ${label}`;
  }
  process.stdout.write(out + "\n");
});
