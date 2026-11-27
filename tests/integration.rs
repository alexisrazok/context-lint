#[path = "../src/analyzer.rs"]
mod analyzer;

use analyzer::{analyze_context, discover_context_files, estimate_tokens};
use std::fs;
use tempfile::TempDir;

#[test]
fn detects_conflicts_duplicates_and_mcp_bloat() {
    let proj = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    fs::write(
        proj.path().join("CLAUDE.md"),
        "- Always use pnpm for package management.\n- Never commit .env files.\n",
    )
    .unwrap();
    fs::write(
        proj.path().join("AGENTS.md"),
        "- Always use npm for package management.\n- Never commit .env files.\n",
    )
    .unwrap();

    let gemini_cfg = home.path().join(".gemini").join("config");
    fs::create_dir_all(&gemini_cfg).unwrap();
    fs::write(
        gemini_cfg.join("mcp_config.json"),
        r#"{"mcpServers":{"github":{"command":"npx","args":[]}}}"#,
    )
    .unwrap();

    let files = discover_context_files(proj.path(), true, Some(home.path()));
    let report = analyze_context(&files, 400, 1500);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();

    assert!(codes.contains(&"DUAL_ROOT_INSTRUCTIONS"));
    assert!(codes.contains(&"CONFLICT_PACKAGE_MANAGER"));
    assert!(codes.contains(&"DUP_RULE"));
    assert!(codes.contains(&"HEAVY_GLOBAL_MCP"));
    assert!(report.summary.est_mcp_schema_tokens >= 4500);
}

#[test]
fn detects_polarity_conflict() {
    let proj = TempDir::new().unwrap();
    fs::write(
        proj.path().join("CLAUDE.md"),
        "- Always run unit tests before commit.\n",
    )
    .unwrap();
    fs::write(
        proj.path().join("GEMINI.md"),
        "- Never run unit tests before commit.\n",
    )
    .unwrap();

    let files = discover_context_files(proj.path(), false, None);
    let report = analyze_context(&files, 400, 1500);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"CONFLICT_POLARITY"));
}

#[test]
fn ignores_code_blocks_and_handles_bloat_thresholds() {
    let proj = TempDir::new().unwrap();
    let big_body = "Always write clean code.\n".repeat(250);
    fs::write(
        proj.path().join("CLAUDE.md"),
        format!("```bash\n# Never run this\n```\n{}", big_body),
    )
    .unwrap();

    let files = discover_context_files(proj.path(), false, None);
    let report = analyze_context(&files, 100, 300);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"BLOAT_ERR"));
    assert!(!codes.contains(&"CONFLICT_POLARITY"));
    assert!(estimate_tokens(&big_body) > 300);
}
