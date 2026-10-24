# TESTS_BACKEND.md — Tests de backend

> Documentación de la suite de tests del backend Rust. **Estado:** MVP `0.1.0`. **Fecha:** 2026-10-22.
>
> Este documento describe **qué prueba cada test**. Las reglas de negocio, el modelo de datos y los contratos son fuente de verdad en `PLAN.md`; aquí solo se documenta su verificación automatizada.
>
> La suite de frontend está documentada por separado en `TESTS_FRONTEND.md`.

---

## 1. Panorama

| Suite | Tests | Archivos | Runner |
|-------|------:|---------:|--------|
| Backend Rust | **34** | | 13 módulos `#[cfg(test)]` | `cargo test` |

- No hay tests de integración en `src-tauri/tests/`.
- Los tests de dominio usan SQLite real en memoria y aplican las migraciones del proyecto.
- Los tests de filesystem, backup y exports usan directorios temporales con `tempfile`.

### 1.1 Cómo ejecutar

```bash
cd src-tauri

cargo test
cargo test -- --list     # listar tests sin ejecutarlos
cargo test modulo::test  # ejecutar un test o módulo puntual
```

### 1.2 Convenciones

- La mayoría de los tests usa SQLite en memoria (`:memory:`) y aplica las migraciones con `db::migrate`.
- Los tests de filesystem, backup y exports usan `tempfile`.
- Los tests asíncronos se ejecutan con `tauri::async_runtime::block_on`.
- Los tests de dominio crean datos reales a través de los `service`, salvo casos puntuales donde se insertan filas para simular estados.
- Los comandos Tauri no se testean directamente: se testea la capa `service`, que es donde viven las reglas y validaciones.

### 1.3 Distribución

| Módulo | Archivo | Tests |
|--------|---------|------:|
| Base de datos | `src-tauri/src/db.rs` | 5 |
| Errores | `src-tauri/src/error.rs` | 2 |
| Estado de arranque | `src-tauri/src/state.rs` | 1 |
| Ajustes | `src-tauri/src/settings.rs` | 5 |
| Utilidades de fechas | `src-tauri/src/util/dates.rs` | 2 |
| Utilidades de archivos | `src-tauri/src/util/files.rs` | 2 |
| Disponibilidad y recomendaciones | `src-tauri/src/util/availability.rs` | 3 |
| Backup y restauración | `src-tauri/src/backup.rs` | 2 |
| Personal | `src-tauri/src/domain/staff/service.rs` | 4 |
| Lugares | `src-tauri/src/domain/places/service.rs` | 2 |
| Servicios de limpieza | `src-tauri/src/domain/cleaning/service.rs` | 2 |
| Incidencias | `src-tauri/src/domain/incidents/service.rs` | 1 |
| Reportes y exports | `src-tauri/src/domain/reports/queries.rs` | 3 |
| **Total** | | **34** |

---

## 2. Infraestructura

### 2.1 `src-tauri/src/db.rs` — 5 tests

| Test | Qué verifica |
|------|--------------|
| `connect_creates_parent_directories` | `connect` crea los directorios padre y el archivo de BD; las migraciones corren correctamente. |
| `migrations_create_the_expected_schema` | Existen las 9 tablas esperadas (`staff`, `place`, `cleaning_service`, `service_staff`, `incident`, `absence`, `staff_place`, `staff_rest_day`, `settings`), 2 filas iniciales en `settings` y el trigger `trg_service_staff_keep_last`. |
| `migrations_use_lf_line_endings` | Las migraciones embebidas no contienen CRLF: evita el error «migration N was previously applied but has been modified» por finales de línea. |
| `valid_database_reopens_without_notice` | Abrir una BD válida dos veces no genera aviso de recuperación. |
| `corrupt_database_is_quarantined_and_recreated` | Un archivo corrupto se pone en cuarentena (`.corrupt-*`), se recrea la BD, se emite un aviso mencionando "corrupt" y "Ajustes", y la tabla `settings` existe de nuevo. |

### 2.2 `src-tauri/src/error.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `serializes_code_and_message` | `CommandError` se serializa como JSON con las claves `code` y `message`. |
| `maps_unique_constraint_to_duplicate_code` | Una violación `UNIQUE` de SQLx se traduce al código de error `duplicate`. |

### 2.3 `src-tauri/src/state.rs` — 1 test

| Test | Qué verifica |
|------|--------------|
| `startup_notice_is_consumed_once` | `StartupNotice` se entrega una sola vez; la segunda lectura devuelve `None`. |

### 2.4 `src-tauri/src/settings.rs` — 5 tests

| Test | Qué verifica |
|------|--------------|
| `fresh_database_returns_defaults` | Una BD recién migrada devuelve `AppSettings::default()`. |
| `settings_are_saved_and_read_back` | Guardar y releer `recent_vacated_days` y `upcoming_occupancy_days` conserva los valores. |
| `out_of_range_values_are_rejected` | Valores fuera de rango (por ejemplo, 31 días) devuelven `settings_invalid`. |
| `invalid_stored_values_fall_back_to_defaults` | Valores corruptos almacenados (`not-a-number`, `99`) caen a los defaults. |
| `missing_keys_fall_back_to_defaults` | Si faltan claves en la tabla `settings`, se usan los defaults. |

### 2.5 `src-tauri/src/util/dates.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `parses_valid_dates_and_rejects_invalid_ones` | Parsea fechas ISO (`2026-10-21`), calcula el día de la semana (miércoles = 2) y rechaza `21/10/2026` con código `validation`. |
| `validates_date_ranges_and_overlaps` | Valida rangos correctos, detecta solapamiento con fecha límite compartida, distingue rangos que no se solapan y rechaza rangos invertidos. |

### 2.6 `src-tauri/src/util/files.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `stores_replaces_and_resolves_photo` | Guarda una foto en `photos/staff/7/photo-*.png`, la resuelve a ruta absoluta; al reemplazarla, la anterior deja de resolverse y la nueva sí se resuelve. |
| `rejects_unsupported_extension` | Una extensión no soportada (`.exe`) devuelve código `validation`. |

### 2.7 `src-tauri/src/util/availability.rs` — 3 tests

| Test | Qué verifica |
|------|--------------|
| `seasonal_and_rest_days_and_absences_govern_availability` | Personal inactivo no está disponible; personal temporal fuera de temporada no está disponible; un día de descanso bloquea la fecha; una ausencia puntual bloquea esas fechas. |
| `recommended_task_follows_priority_rules` | Prioridad de recomendaciones: override `cancelled` → sin recomendación; checkout hoy → `recently_vacated` + `deep_cleaning`; check-in próximo → `upcoming_occupancy` + `touch_up`; ocupado → `occupied` + `daily_cleaning`; fechas parciales; sin fechas → `vacant`; override no cancelado conserva la recomendación por fechas. |
| `ranks_candidates_by_role_main_place_rotation_and_id` | Orden de candidatos (rol + apartamento principal → menos servicios recientes → más recientes → otro rol), exclusión por solapamiento horario y presencia de motivos ("Apartamento principal", "Advertencia"). |

---

## 3. Dominio

### 3.1 `src-tauri/src/domain/staff/service.rs` — 4 tests

| Test | Qué verifica |
|------|--------------|
| `staff_crud_and_national_id_uniqueness` | Alta de personal; cédula duplicada con personal activo → `duplicate`; baja; alta con cédula de personal inactivo → `duplicate` con mensaje que sugiere "reactivarlo"; reactivación. |
| `seasonal_validation_rules` | Personal no temporal con fechas → error; temporal con una sola fecha → error; fechas invertidas → error; temporal sin fechas → permitido (marca informativa); temporal con fechas válidas → permitido. |
| `rest_days_and_main_places_and_absences_rules` | Los días de descanso se deduplican y ordenan; el día 7 es inválido; solo se aceptan apartamentos activos como lugar principal (rechaza áreas comunes e inactivos); una ausencia que toca el fin de otra → `duplicate`; editar la propia ausencia extendiéndola → permitido; borrar ausencia. |
| `deactivate_blocks_when_sole_staff_on_future_service_and_warns_when_shared` | La baja se bloquea (`blocked`) si la persona es la única asignada a un servicio futuro; se permite con `warning` si hay otra persona asignada. |

### 3.2 `src-tauri/src/domain/places/service.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `place_crud_and_case_insensitive_uniqueness` | Alta de apartamento; duplicado ignorando mayúsculas → `duplicate`; área común con estado → `validation`; área común válida. |
| `recommended_task_and_cancelled_status_and_deactivation_block` | Recomendación `occupied` → limpieza diaria; checkout dentro de la ventana → `recently_vacated` + limpieza profunda; override `cancelled` → sin recomendación; un servicio futuro activo bloquea la baja; al cancelarlo la baja procede; reactivación. |

### 3.3 `src-tauri/src/domain/cleaning/service.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `service_crud_mandatory_staff_and_transitions` | Servicio sin personal → error; dos responsables → error; alta válida; reemplazo del único asignado sin disparar el trigger de "mantener al menos uno"; `in_progress` exige hora de inicio; `completed` exige fin > inicio; reabrir un servicio completado se bloquea sin confirmación y se permite con ella. |
| `assisted_staff_suggestions_respect_availability_and_ranking` | Las sugerencias excluyen al personal ausente y ordenan: maid con apartamento principal → maid sin él → limpiacristales. |

### 3.4 `src-tauri/src/domain/incidents/service.rs` — 1 test

| Test | Qué verifica |
|------|--------------|
| `creates_lists_resolves_and_rejects_cancelled_service_incidents` | Descripción vacía → `validation`; alta en servicio completado con estado `open`, nombre del lugar y sin `resolved_at`; resolver marca estado y fecha; reabrir limpia la fecha; no se permiten incidencias sobre un servicio cancelado. |

### 3.5 `src-tauri/src/domain/reports/queries.rs` — 3 tests

| Test | Qué verifica |
|------|--------------|
| `services_by_staff_and_average_times_exclude_cancelled_and_compute_metrics` | Conteo y horas por persona excluyendo cancelados (Ana: 2 servicios / 2 h; Beatriz: 1 / 0.5 h); promedio de `daily_cleaning` = 60 min; `daily_sheet` devuelve duración (90 min), incidencias abiertas y nombres con "(Responsable)". |
| `absences_by_month_prorates_cross_month_ranges` | Una ausencia del 28/09 al 03/10 se reparte proporcionalmente: 3 días en septiembre; octubre suma 3 + 2 = 5 días. |
| `exports_csv_xlsx_and_pdf_generate_valid_non_empty_files` | Para los 4 reportes genera CSV, XLSX y PDF: CSV con BOM UTF-8 y encabezado; XLSX con firma `PK` y más de 100 bytes; PDF con firma `%PDF-` y más de 200 bytes; incluye texto con ñ y acentos. |

---

## 4. Backup y restauración

### 4.1 `src-tauri/src/backup.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `backup_and_restore_round_trip_preserves_database_and_photos` | Crea datos (personal, lugar y foto), genera un ZIP de respaldo, modifica la BD viva y borra la foto, y luego restaura: el manifiesto reporta los conteos originales, la BD vuelve a tener solo el lugar original y la foto se recupera con su contenido intacto. |
| `restore_rejects_corrupt_zip_and_keeps_live_database_intact` | Rechaza un ZIP corrupto y un ZIP válido cuyo `housekeeping.db` no es SQLite, ambos con código `invalid_backup`; la BD viva conserva sus datos. |

---

## 5. Cobertura y huecos

### 5.1 Bien cubierto

- Reglas de negocio: disponibilidad, recomendaciones, validaciones, transiciones de estado, bloqueos por integridad, unicidad y solapamientos.
- Migraciones, esquema, trigger de integridad y recuperación ante BD corrupta.
- Backup/restauración completo (datos + fotos) y rechazo de respaldos inválidos.
- Cálculo de métricas de reportes y generación de archivos CSV/XLSX/PDF.

### 5.2 Sin cobertura (por decisión)

- **Comandos Tauri (`commands.rs`):** son adaptadores finos; la lógica está cubierta en `service.rs`.
- **Capa IPC real:** se testea el servicio, no el puente Tauri.
- **Contenido exacto de los exports:** se valida formato, firma y tamaño, no cada celda o fila.
- **Migraciones históricas:** se aplica el esquema vigente, no se testea la migración incremental desde versiones antiguas.
- **Concurrencia real:** la BD usa `max_connections(1)`; no se simulan múltiples procesos.

### 5.3 Posibles mejoras

1. Añadir una suite de integración en `src-tauri/tests/` que ejercite comandos Tauri completos.
2. Verificar el contenido de los reportes exportados, no solo su validez formal.
3. Probar migraciones incrementales si se agregan nuevas versiones del esquema.
4. Ejecutar `cargo test` en CI para Windows, Linux y macOS.
