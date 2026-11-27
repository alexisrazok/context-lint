# Security Policy

## Reporting a Vulnerability

If you discover a security issue in `context-lint` (for example, symlink traversal, path handling issues, or denial-of-service via malformed configuration files), please open a private security advisory on GitHub rather than a public issue.

## Security Design

- `context-lint` is strictly read-only and never executes shell commands or modifies repository files.
- Files larger than 10 MiB are skipped to prevent memory exhaustion.
- Symlink canonicalization deduplicates files safely without following infinite directory loops.
