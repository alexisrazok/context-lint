use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB safety cap
const DEFAULT_MCP_OVERHEAD: usize = 1200;

const INSTRUCTION_PATTERNS: &[&str] = &[
    "CLAUDE.md",
    "CLAUDE.local.md",
    "AGENTS.md",
    "GEMINI.md",
    ".cursorrules",
    ".github/copilot-instructions.md",
];

const MCP_PROJECT_FILES: &[&str] = &[".mcp.json", ".vscode/mcp.json", ".cursor/mcp.json"];

#[derive(Debug, Clone, Serialize)]
pub struct ContextFile {
    pub path: String,
    pub scope: String,
    pub kind: String,
    pub lines: usize,
    pub est_tokens: usize,
    #[serde(skip)]
    pub content: String,
    #[serde(skip)]
    pub is_symlink: bool,
    #[serde(skip)]
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpServerInfo {
    pub server: String,
    pub scope: String,
    pub est_tokens: usize,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub instruction_files: usize,
    pub skill_files: usize,
    pub mcp_servers: usize,
    pub est_instruction_tokens: usize,
    pub est_skill_metadata_tokens: usize,
    pub est_mcp_schema_tokens: usize,
    pub est_total_startup_tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintReport {
    pub files: Vec<ContextFile>,
    pub mcp_servers: Vec<McpServerInfo>,
    pub summary: Summary,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone)]
struct Directive {
    source: String,
    line_no: usize,
    raw: String,
    polarity: char,
    normalized: String,
}

pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let char_est = text.chars().count() as f64 / 4.0;
    let word_est = text.split_whitespace().count() as f64 * 1.25;
    ((char_est + word_est) / 2.0).floor().max(1.0) as usize
}

fn read_text_capped(path: &Path) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn track_file(
    out: &mut Vec<ContextFile>,
    seen: &mut HashSet<PathBuf>,
    path: PathBuf,
    scope: &str,
    kind: &str,
) {
    let is_symlink = fs::symlink_metadata(&path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    let canonical = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    if seen.contains(&canonical) && !is_symlink {
        return;
    }
    if let Some(content) = read_text_capped(&path) {
        seen.insert(canonical);
        let lines = content.lines().count();
        let est_tokens = estimate_tokens(&content);
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        out.push(ContextFile {
            path: path.to_string_lossy().into_owned(),
            scope: scope.to_string(),
            kind: kind.to_string(),
            lines,
            est_tokens,
            content,
            is_symlink,
            file_name,
        });
    }
}

fn scan_skills_dir(
    out: &mut Vec<ContextFile>,
    seen: &mut HashSet<PathBuf>,
    dir: &Path,
    scope: &str,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("SKILL.md"))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    for skill_md in paths {
        track_file(out, seen, skill_md, scope, "skill");
    }
}

pub fn discover_context_files(
    project_root: &Path,
    include_global: bool,
    home_override: Option<&Path>,
) -> Vec<ContextFile> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    for rel in INSTRUCTION_PATTERNS {
        let mut p = project_root.to_path_buf();
        for seg in rel.split('/') {
            p.push(seg);
        }
        track_file(&mut files, &mut seen, p, "project", "instructions");
    }

    let cursor_rules = project_root.join(".cursor").join("rules");
    if let Ok(entries) = fs::read_dir(&cursor_rules) {
        let mut rule_paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        rule_paths.sort();
        for rp in rule_paths {
            track_file(&mut files, &mut seen, rp, "project", "instructions");
        }
    }

    scan_skills_dir(
        &mut files,
        &mut seen,
        &project_root.join(".agents").join("skills"),
        "project",
    );
    scan_skills_dir(
        &mut files,
        &mut seen,
        &project_root.join(".claude").join("skills"),
        "project",
    );

    for rel in MCP_PROJECT_FILES {
        let mut p = project_root.to_path_buf();
        for seg in rel.split('/') {
            p.push(seg);
        }
        track_file(&mut files, &mut seen, p, "project", "mcp");
    }

    if include_global {
        let home = home_override.map(Path::to_path_buf).or_else(dirs::home_dir);
        if let Some(base) = home {
            let global_instr = [
                base.join(".claude").join("CLAUDE.md"),
                base.join(".gemini").join("config").join("GEMINI.md"),
                base.join(".codex").join("AGENTS.md"),
            ];
            for gp in global_instr {
                track_file(&mut files, &mut seen, gp, "global", "instructions");
            }

            let global_mcps = [
                base.join(".gemini").join("config").join("mcp_config.json"),
                base.join(".claude.json"),
                base.join(".claude").join("settings.json"),
                base.join(".codex").join("config.toml"),
                base.join(".cursor").join("mcp.json"),
            ];
            for mp in global_mcps {
                track_file(&mut files, &mut seen, mp, "global", "mcp");
            }

            for sr in [
                base.join(".agents").join("skills"),
                base.join(".gemini").join("config").join("skills"),
                base.join(".claude").join("skills"),
                base.join(".codex").join("skills"),
            ] {
                scan_skills_dir(&mut files, &mut seen, &sr, "global");
            }
        }
    }

    files
}

fn extract_directives(text: &str, source: &str) -> Vec<Directive> {
    let pos_re =
        Regex::new(r"(?i)\b(always|use|prefer|must|require|siempre|usa|utiliza|prefiere|debes)\b")
            .unwrap();
    let neg_re = Regex::new(r"(?i)\b(never|do not|don't|avoid|forbid|prohibit|nunca|no uses|no utilices|evita|prohibido)\b").unwrap();
    let prefix_re = Regex::new(r"^([-*+]\s+|\d+\.\s+)").unwrap();
    let clean_re = Regex::new(r"[^a-z0-9áéíóúñ\s]").unwrap();
    let ws_re = Regex::new(r"\s+").unwrap();

    let mut out = Vec::new();
    let mut in_code_block = false;
    for (idx, line) in text.lines().enumerate() {
        let stripped = line.trim();
        if stripped.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        let clean = prefix_re.replace(stripped, "").to_string();
        if clean.chars().count() < 8 {
            continue;
        }
        let is_neg = neg_re.is_match(&clean);
        let is_pos = pos_re.is_match(&clean);
        if !is_neg && !is_pos {
            continue;
        }
        let polarity = if is_neg { '-' } else { '+' };
        let lowered = clean.to_lowercase();
        let stripped_punct = clean_re.replace_all(&lowered, " ");
        let normalized = ws_re.replace_all(&stripped_punct, " ").trim().to_string();
        out.push(Directive {
            source: source.to_string(),
            line_no: idx + 1,
            raw: clean,
            polarity,
            normalized,
        });
    }
    out
}

fn extract_mcp_servers(cf: &ContextFile) -> Vec<String> {
    if cf.content.trim().is_empty() {
        return Vec::new();
    }
    if cf.path.ends_with(".toml") {
        let Ok(val) = toml::from_str::<toml::Value>(&cf.content) else {
            return Vec::new();
        };
        if let Some(table) = val.get("mcp_servers").and_then(|v| v.as_table()) {
            return table.keys().cloned().collect();
        }
        return Vec::new();
    }
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&cf.content) else {
        return Vec::new();
    };
    let servers = val.get("mcpServers").or_else(|| val.get("servers"));
    if let Some(obj) = servers.and_then(|v| v.as_object()) {
        return obj.keys().cloned().collect();
    }
    Vec::new()
}

fn heavy_mcp_hint(name: &str) -> (usize, Option<&'static str>) {
    let lower = name.to_lowercase();
    let table: &[(&str, usize, &str)] = &[
        (
            "github",
            4500,
            "Consider using `gh` CLI in shell instead of loading full GitHub MCP schemas.",
        ),
        (
            "playwright",
            3800,
            "Playwright MCP exposes many browser tools; enable only in UI-testing projects.",
        ),
        (
            "figma",
            3200,
            "Figma MCP schemas are large; scope to design/frontend projects.",
        ),
        (
            "notion",
            2800,
            "Notion MCP adds significant schema overhead on every turn.",
        ),
        (
            "linear",
            2400,
            "Consider `linear` CLI or project-scoped MCP instead of global.",
        ),
        (
            "filesystem",
            1800,
            "Most coding agents already have native file tools; filesystem MCP is often redundant.",
        ),
    ];
    for (pat, cost, hint) in table {
        if lower.contains(pat) {
            return (*cost, Some(*hint));
        }
    }
    (DEFAULT_MCP_OVERHEAD, None)
}

pub fn analyze_context(
    files: &[ContextFile],
    max_tokens_warn: usize,
    max_tokens_err: usize,
) -> LintReport {
    let mut findings = Vec::new();
    let mut directives = Vec::new();
    let mut mcp_summary = Vec::new();

    let instr_files: Vec<&ContextFile> =
        files.iter().filter(|f| f.kind == "instructions").collect();
    let skill_files: Vec<&ContextFile> = files.iter().filter(|f| f.kind == "skill").collect();
    let mcp_files: Vec<&ContextFile> = files.iter().filter(|f| f.kind == "mcp").collect();

    for cf in &instr_files {
        if cf.est_tokens >= max_tokens_err {
            findings.push(Finding {
                severity: "error".into(),
                code: "BLOAT_ERR".into(),
                message: format!(
                    "{} ({}) is very large ({} est. tokens, {} lines). Split or move details to skills.",
                    cf.file_name, cf.scope, cf.est_tokens, cf.lines
                ),
                files: vec![cf.path.clone()],
            });
        } else if cf.est_tokens >= max_tokens_warn {
            findings.push(Finding {
                severity: "warning".into(),
                code: "BLOAT_WARN".into(),
                message: format!(
                    "{} ({}) exceeds recommended {} tokens ({} est. tokens, {} lines).",
                    cf.file_name, cf.scope, max_tokens_warn, cf.est_tokens, cf.lines
                ),
                files: vec![cf.path.clone()],
            });
        }
        directives.extend(extract_directives(&cf.content, &cf.path));
    }

    let proj_claude = instr_files
        .iter()
        .find(|f| f.scope == "project" && f.file_name == "CLAUDE.md");
    let proj_agents = instr_files
        .iter()
        .find(|f| f.scope == "project" && f.file_name == "AGENTS.md");
    if let (Some(cf), Some(af)) = (proj_claude, proj_agents) {
        if !cf.is_symlink && !af.is_symlink {
            findings.push(Finding {
                severity: "warning".into(),
                code: "DUAL_ROOT_INSTRUCTIONS".into(),
                message: "Both CLAUDE.md and AGENTS.md exist as separate files. Symlink or reference one from the other to avoid drift.".into(),
                files: vec![cf.path.clone(), af.path.clone()],
            });
        }
    }

    let cue_prefix_re =
        Regex::new(r"^(always|never|do not|don t|use|prefer|avoid|siempre|nunca|usa|evita)\s+")
            .unwrap();
    let mut seen_norm: HashMap<String, Directive> = HashMap::new();
    for d in &directives {
        let core = cue_prefix_re.replace(&d.normalized, "").to_string();
        if core.chars().count() < 12 {
            continue;
        }
        if let Some(prev) = seen_norm.get(&core) {
            if prev.source != d.source {
                if prev.polarity == d.polarity {
                    findings.push(Finding {
                        severity: "info".into(),
                        code: "DUP_RULE".into(),
                        message: format!(
                            "Duplicate rule in two files: \"{}\"",
                            d.raw.chars().take(70).collect::<String>()
                        ),
                        files: vec![
                            format!("{}:{}", prev.source, prev.line_no),
                            format!("{}:{}", d.source, d.line_no),
                        ],
                    });
                } else {
                    findings.push(Finding {
                        severity: "error".into(),
                        code: "CONFLICT_POLARITY".into(),
                        message: format!(
                            "Conflicting rules on same instruction: \"{}\" vs \"{}\"",
                            prev.raw.chars().take(55).collect::<String>(),
                            d.raw.chars().take(55).collect::<String>()
                        ),
                        files: vec![
                            format!("{}:{}", prev.source, prev.line_no),
                            format!("{}:{}", d.source, d.line_no),
                        ],
                    });
                }
            }
        } else {
            seen_norm.insert(core, d.clone());
        }
    }

    type ExclusiveGroup<'a> = (&'a str, &'a [(&'a str, &'a [&'a str])]);
    let exclusive_groups: &[ExclusiveGroup<'_>] = &[
        (
            "package_manager",
            &[
                ("npm", &["npm"]),
                ("pnpm", &["pnpm"]),
                ("yarn", &["yarn"]),
                ("bun", &["bun"]),
            ],
        ),
        (
            "python_env",
            &[
                ("uv", &["uv", "uvx"]),
                ("poetry", &["poetry"]),
                ("pipenv", &["pipenv"]),
            ],
        ),
        (
            "indentation",
            &[
                ("tabs", &["tabs", "tab indentation"]),
                ("spaces", &["spaces", "2 spaces", "4 spaces"]),
            ],
        ),
        (
            "response_language",
            &[
                ("spanish", &["spanish", "español", "castellano"]),
                ("english", &["english", "inglés"]),
            ],
        ),
        (
            "js_typing",
            &[
                ("typescript", &["typescript"]),
                ("javascript", &["plain javascript", "vanilla javascript"]),
            ],
        ),
    ];

    for (group_name, options) in exclusive_groups {
        let mut matched: HashMap<&str, Vec<&Directive>> = HashMap::new();
        for d in directives.iter().filter(|d| d.polarity == '+') {
            for (opt_name, keywords) in *options {
                let hit = keywords.iter().any(|kw| {
                    let pat = format!(r"\b{}\b", regex::escape(kw));
                    Regex::new(&pat)
                        .map(|r| r.is_match(&d.normalized))
                        .unwrap_or(false)
                });
                if hit {
                    matched.entry(opt_name).or_default().push(d);
                }
            }
        }
        if matched.len() > 1 {
            let mut choices: Vec<&str> = matched.keys().copied().collect();
            choices.sort();
            let refs: Vec<String> = choices
                .iter()
                .filter_map(|c| matched.get(c).and_then(|v| v.first()))
                .map(|d| format!("{}:{}", d.source, d.line_no))
                .collect();
            findings.push(Finding {
                severity: "error".into(),
                code: format!("CONFLICT_{}", group_name.to_uppercase()),
                message: format!(
                    "Conflicting {} preferences across instructions: {}.",
                    group_name,
                    choices.join(", ")
                ),
                files: refs,
            });
        }
    }

    let mut total_mcp_tokens = 0usize;
    for mf in &mcp_files {
        for srv in extract_mcp_servers(mf) {
            let (est, hint) = heavy_mcp_hint(&srv);
            total_mcp_tokens += est;
            mcp_summary.push(McpServerInfo {
                server: srv.clone(),
                scope: mf.scope.clone(),
                est_tokens: est,
                file: mf.path.clone(),
            });
            if let Some(advice) = hint {
                if mf.scope == "global" {
                    findings.push(Finding {
                        severity: "warning".into(),
                        code: "HEAVY_GLOBAL_MCP".into(),
                        message: format!(
                            "Global MCP server '{}' adds ~{} est. tokens per session. {}",
                            srv, est, advice
                        ),
                        files: vec![mf.path.clone()],
                    });
                }
            }
        }
    }

    let total_instr_tokens: usize = instr_files.iter().map(|f| f.est_tokens).sum();
    let total_skill_meta_tokens = skill_files.len() * 80;

    LintReport {
        files: files.to_vec(),
        mcp_servers: mcp_summary,
        summary: Summary {
            instruction_files: instr_files.len(),
            skill_files: skill_files.len(),
            mcp_servers: 0, // updated below
            est_instruction_tokens: total_instr_tokens,
            est_skill_metadata_tokens: total_skill_meta_tokens,
            est_mcp_schema_tokens: total_mcp_tokens,
            est_total_startup_tokens: total_instr_tokens
                + total_skill_meta_tokens
                + total_mcp_tokens,
        },
        findings,
    }
    .with_mcp_count()
}

impl LintReport {
    fn with_mcp_count(mut self) -> Self {
        self.summary.mcp_servers = self.mcp_servers.len();
        self
    }
}
