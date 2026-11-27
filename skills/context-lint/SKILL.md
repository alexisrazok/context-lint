---
name: context-lint
description: Automatically audit AI agent context files (CLAUDE.md, AGENTS.md, GEMINI.md, Cursor rules, skills, and MCP configs) for token bloat, duplicate rules, and conflicting instructions. Trigger whenever editing agent instruction files, adding MCP servers, or debugging why an agent ignores rules.
---

# context-lint

Use `context-lint` automatically to keep startup context lean and conflict-free across all AI coding agents.

## When to Run Automatically

1. **After editing** `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursorrules`, `.cursor/rules/*`, or any MCP config (`.mcp.json`, `mcp_config.json`, `config.toml`).
2. **Before adding a new global MCP server** to verify startup token overhead.
3. **When an agent behaves inconsistently** across tools or seems to ignore instructions.

## Workflow

1. Check if `context-lint` is installed (`context-lint --version`). If missing, install via `cargo install --git https://github.com/alexisrazok/context-lint` or download from GitHub Releases.
2. Run an audit on the current repository:
   ```bash
   context-lint .
   ```
3. For CI/automated checks or structured parsing:
   ```bash
   context-lint . --json
   context-lint . --no-global --strict
   ```

## How to Act on Findings

- **`DUAL_ROOT_INSTRUCTIONS`**: Suggest running `dotagents unify` or symlinking `CLAUDE.md` and `AGENTS.md`.
- **`CONFLICT_*` / `CONFLICT_POLARITY`**: Surface both conflicting lines to the user and ask which rule should prevail.
- **`BLOAT_WARN` / `BLOAT_ERR`**: Offer to split detailed reference material out of root instruction files into on-demand skills.
- **`HEAVY_GLOBAL_MCP`**: Recommend scoping heavy MCPs (`github`, `playwright`, `figma`) to specific projects or using native CLIs (e.g., `gh`) instead.
