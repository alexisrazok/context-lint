# context-lint

[![CI](https://github.com/alexisrazok/context-lint/actions/workflows/ci.yml/badge.svg)](https://github.com/alexisrazok/context-lint/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/alexisrazok/context-lint)](https://github.com/alexisrazok/context-lint/releases)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Linter rápido y multiplataforma escrito en Rust para auditar el contexto de arranque de agentes de código (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursor/rules`, skills y servidores MCP).

## El problema que resuelve

Cuando trabajas con varios agentes de código en un proyecto, los archivos de instrucciones (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursor/rules`) y los servidores MCP globales van acumulando texto y esquemas sin control. Eso provoca tres problemas frecuentes:

1. **Gasto silencioso de tokens desde el primer turno**: un archivo de instrucciones largo o varios MCPs globales cargan miles de tokens antes de que escribas tu primera pregunta.
2. **Reglas contradictorias**: un archivo pide `pnpm` y otro `npm`, o uno dice que siempre hagas tests y otro lo contrario, haciendo que el agente ignore instrucciones sin motivo aparente.
3. **Deriva entre archivos**: mantener `CLAUDE.md` y `AGENTS.md` por separado acaba generando versiones distintas del mismo proyecto.

## Qué detecta

- **Consumo de tokens al arrancar**: estima cuántos tokens cargan tus archivos de instrucciones, metadatos de skills y esquemas de servidores MCP activos.
- **Reglas contradictorias**: detecta conflictos entre archivos (`use npm` frente a `use pnpm`, `tabs` frente a `spaces`, idioma, polaridad opuesta en una misma regla).
- **Reglas duplicadas y drift**: avisa si tienes `CLAUDE.md` y `AGENTS.md` separados sin enlazar o reglas repetidas en varios archivos.
- **Sobrecarga de MCP globales**: identifica servidores MCP pesados cargados globalmente (`github`, `playwright`, `figma`, `notion`, `linear`, `filesystem`).

## Instalación

### Desde crates.io con Cargo
```bash
cargo install context-lint
```

### Binarios precompilados (Linux, macOS, Windows)
Descarga el binario correspondiente a tu sistema operativo desde la página de [Releases](https://github.com/alexisrazok/context-lint/releases/latest).

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

## FAQ

**¿Modifica o borra algún archivo de mi repositorio?**
No. `context-lint` es estrictamente de solo lectura: analiza los archivos y muestra el informe en pantalla o en JSON.

**¿Envía el contenido de mis archivos a algún servidor o API externa?**
No. Todo el análisis y el cálculo estimado de tokens se ejecutan en local.

**¿Cómo soluciono un aviso `DUAL_ROOT_INSTRUCTIONS` o conflictos entre reglas?**
Puedes usar [dotagents](https://github.com/alexisrazok/dotagents) (`dotagents unify`) para unificar y deduplicar tus archivos existentes desde una única fuente de verdad.

## Desarrollo, Seguridad y Licencia

- Guía de contribución: [CONTRIBUTING.md](CONTRIBUTING.md)
- Política de seguridad: [SECURITY.md](SECURITY.md)
- Historial de cambios: [CHANGELOG.md](CHANGELOG.md)
- Licencia: [Apache-2.0](LICENSE) y [NOTICE](NOTICE)
