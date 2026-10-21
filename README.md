# Gestión de Limpieza

Aplicación de escritorio para gestionar el personal de limpieza y los servicios de un edificio. Funciona **offline**, en un solo equipo, con una única persona encargada.

- **Interfaz:** 100 % en español.
- **Código:** identificadores, tablas, rutas y archivos en inglés.
- **Datos:** SQLite local + fotos en la carpeta de datos de la aplicación.

## Estado

**Fase 0 — Fundación completada.** El tablero y la gestión de personal, lugares, servicios, incidencias, reportes y respaldos se implementan en las fases siguientes. Ver `PLAN.md` §8.

## Stack

| Capa | Tecnología |
|------|------------|
| Escritorio | Tauri v2 (Rust) |
| Frontend | Svelte 5 + SvelteKit 3 (SPA estática) |
| Estilos | CSS nativo con design tokens |
| Base de datos | SQLite + SQLx 0.8 (migraciones versionadas) |
| Runtime / paquetes | Bun 1.4.3 |

## Requisitos

- [Bun](https://bun.sh/) 1.4.3 o superior.
- [Rust](https://www.rust-lang.org/tools/install) 1.96 o superior (edición 2024).
- Windows: WebView2 (incluido en Windows 11; instalable en Windows 10).

## Comandos

```bash
bun install          # dependencias del frontend
bun run tauri dev    # aplicación completa en modo desarrollo
bun run build        # build del frontend
bun run check        # svelte-check

cd src-tauri
cargo test
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Documentación

- `PLAN.md` — alcance, modelo de datos, reglas de negocio y roadmap.
- `AGENTS.md` — guía operativa y técnica para agentes.
- `CHANGELOG.md` — cambios posteriores al MVP (Keep a Changelog + SemVer).
