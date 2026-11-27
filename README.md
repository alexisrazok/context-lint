# context-lint

Linter rápido y multiplataforma escrito en Rust para auditar el contexto de arranque de agentes de código (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursor/rules`, skills y servidores MCP).

## Qué detecta

- **Consumo de tokens al arrancar**: estima cuántos tokens cargan tus archivos de instrucciones, metadatos de skills y esquemas de servidores MCP activos.
- **Reglas contradictorias**: detecta conflictos entre archivos (`use npm` frente a `use pnpm`, `tabs` frente a `spaces`, idioma, polaridad opuesta en una misma regla).
- **Reglas duplicadas y drift**: avisa si tienes `CLAUDE.md` y `AGENTS.md` separados sin enlazar o reglas repetidas en varios archivos.
- **Sobrecarga de MCP globales**: identifica servidores MCP pesados cargados globalmente (`github`, `playwright`, `figma`, `notion`, `linear`, `filesystem`).

## Instalación

Desde código fuente con Cargo:
```bash
cargo install --git https://github.com/alexisrazok/context-lint
```

O compilando localmente:
```bash
cargo build --release
```

## Uso

```bash
# Auditar el proyecto actual (incluyendo configuración global)
context-lint .

# Solo archivos del proyecto
context-lint . --no-global

# Salida JSON y modo estricto para CI/CD (sale con código 1 si hay errores)
context-lint . --json
context-lint . --strict
```

## Tests

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```
