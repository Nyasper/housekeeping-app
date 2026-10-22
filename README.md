# Gestión de Limpieza

Aplicación de escritorio para gestionar el personal de limpieza y los servicios de un edificio. Funciona **offline**, en un solo equipo, con una única persona encargada.

- **Interfaz:** 100 % en español.
- **Código:** identificadores, tablas, rutas y archivos en inglés.
- **Datos:** SQLite local + fotos en la carpeta de datos de la aplicación.

## Estado

**MVP `0.1.0` completado (Fases 0 a 7).** Incluye:

- **Tablero diario (`/`):** vista operativa por fecha con tareas recomendadas pendientes, servicios del día, disponibilidad del personal e incidencias abiertas/resueltas.
- **Personal (`/staff`):** alta, edición, baja lógica y reactivación de mucamas, vidrieros y personal de áreas comunes; contratos permanentes o zafrales con temporada; descansos semanales, ausencias sin solapamientos, foto local, avatares por género y apartamentos principales.
- **Lugares (`/places`):** apartamentos y áreas comunes con fechas opcionales de ingreso/egreso, estado calculado o manual (`cancelled` para omitir turno) y recomendación automática de tarea.
- **Servicios de limpieza (`/cleaning`):** planificación y seguimiento (`planned`, `in_progress`, `completed`, `cancelled`), personal obligatorio (`>= 1` y responsable opcional) y sugerencia asistida de candidatos según disponibilidad, apartamento principal, rol y carga del día.
- **Incidencias (`/incidents`):** registro y resolución de daños, objetos olvidados y faltantes de insumos asociados a un servicio, con evidencia fotográfica opcional.
- **Reportes y exportación (`/reports`):** servicios y horas por persona, tiempos promedio por tipo de servicio, ausencias mensuales y planilla diaria, exportables a **CSV**, **XLSX** y **PDF** generados en Rust.
- **Ajustes y respaldos (`/settings`):** configuración de ventanas de recomendación (`recent_vacated_days`, `upcoming_occupancy_days`) y respaldo/restauración en archivo **ZIP** (base de datos consistente con `VACUUM INTO` + fotos + manifiesto, validación de integridad y reemplazo en caliente).

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
