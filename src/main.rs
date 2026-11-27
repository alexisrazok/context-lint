mod analyzer;

use analyzer::{analyze_context, discover_context_files, LintReport};
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "context-lint",
    version,
    about = "Audit AI agent context files, token budgets, MCP bloat, and conflicting rules."
)]
struct Cli {
    /// Project directory to audit
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Ignore global (~/.claude, ~/.gemini, ~/.codex) configs
    #[arg(long)]
    no_global: bool,

    /// Warning threshold for instruction files in estimated tokens
    #[arg(long, default_value_t = 400)]
    max_tokens: usize,

    /// Error threshold for instruction files in estimated tokens
    #[arg(long, default_value_t = 1500)]
    max_tokens_err: usize,

    /// Output JSON report
    #[arg(long)]
    json: bool,

    /// Exit with code 1 if any error-level findings exist
    #[arg(long)]
    strict: bool,
}

fn print_report(report: &LintReport) {
    let s = &report.summary;
    println!("context-lint v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", "=".repeat(60));
    println!(
        "Startup context estimate: ~{} tokens (instructions: {}, skills meta: {}, MCP schemas: {})",
        s.est_total_startup_tokens,
        s.est_instruction_tokens,
        s.est_skill_metadata_tokens,
        s.est_mcp_schema_tokens
    );

    let instr: Vec<_> = report
        .files
        .iter()
        .filter(|f| f.kind == "instructions")
        .collect();
    if !instr.is_empty() {
        println!("\nInstruction files:");
        for f in instr {
            println!(
                "  [{:7}] {:5} tok | {:4} lines | {}",
                f.scope, f.est_tokens, f.lines, f.path
            );
        }
    }

    if !report.mcp_servers.is_empty() {
        println!("\nMCP servers ({}):", report.mcp_servers.len());
        for m in &report.mcp_servers {
            println!(
                "  [{:7}] ~{:4} tok | {} ({})",
                m.scope, m.est_tokens, m.server, m.file
            );
        }
    }

    println!("\nFindings ({}):", report.findings.len());
    if report.findings.is_empty() {
        println!("  No issues found.");
        return;
    }
    for fd in &report.findings {
        println!(
            "  [{:7}] {}: {}",
            fd.severity.to_uppercase(),
            fd.code,
            fd.message
        );
        for r in &fd.files {
            println!("            -> {}", r);
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = std::fs::canonicalize(&cli.path).unwrap_or(cli.path);
    let files = discover_context_files(&root, !cli.no_global, None);
    let report = analyze_context(&files, cli.max_tokens, cli.max_tokens_err);

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into())
        );
    } else {
        print_report(&report);
    }

    if cli.strict && report.findings.iter().any(|f| f.severity == "error") {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
