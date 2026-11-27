---
name: context-lint
description: Audit project and global AI agent context files (CLAUDE.md, AGENTS.md, GEMINI.md, Cursor rules, skills, and MCP servers) for token bloat, duplicate rules, and conflicting instructions.
---

# context-lint Skill

Run `context-lint` when setting up a repository, debugging why an agent ignores instructions, or reducing token usage at session startup.

## Commands

```bash
context-lint .
context-lint . --no-global --strict
context-lint . --json
```
