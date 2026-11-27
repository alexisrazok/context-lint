# context-lint

[![CI](https://github.com/alexisrazok/context-lint/actions/workflows/ci.yml/badge.svg)](https://github.com/alexisrazok/context-lint/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/alexisrazok/context-lint)](https://github.com/alexisrazok/context-lint/releases)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Linter rápido y multiplataforma escrito en Rust para auditar el contexto de arranque de agentes de código (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursor/rules`, skills y servidores MCP).

## Qué detecta

- **Consumo de tokens al arrancar**: estima cuántos tokens cargan tus archivos de instrucciones, metadatos de skills y esquemas de servidores MCP activos.
- **Reglas contradictorias**: detecta conflictos entre archivos (`use npm` frente a `use pnpm`, `tabs` frente a `spaces`, idioma, polaridad opuesta en una misma regla).
- **Reglas duplicadas y drift**: avisa si tienes `CLAUDE.md` y `AGENTS.md` separados sin enlazar o reglas repetidas en varios archivos.
- **Sobrecarga de MCP globales**: identifica servidores MCP pesados cargados globalmente (`github`, `playwright`, `figma`, `notion`, `linear`, `filesystem`).

## Instalación

### Binarios precompilados (Linux, macOS, Windows)
Descarga el binario correspondiente a tu sistema operativo desde la página de [Releases](https://github.com/alexisrazok/context-lint/releases/latest).

### Desde código fuente con Cargo
```bash
cargo install --git https://github.com/alexisrazok/context-lint
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

## Desarrollo, Seguridad y Licencia

- Guía de contribución: [CONTRIBUTING.md](CONTRIBUTING.md)
- Política de seguridad: [SECURITY.md](SECURITY.md)
- Historial de cambios: [CHANGELOG.md](CHANGELOG.md)
- Licencia: [Apache-2.0](LICENSE) y [NOTICE](NOTICE)
