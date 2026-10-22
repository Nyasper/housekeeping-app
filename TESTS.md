# TESTS.md — Documentación de tests

> Resumen de las suites de tests del proyecto. **Estado:** MVP `0.1.0`. **Fecha:** 2026-10-22.
>
> Este documento describe **qué prueba cada test**. Las reglas de negocio y el modelo de datos son fuente de verdad en `PLAN.md`; aquí solo se documenta su verificación automatizada.

---

## 1. Panorama general

| Suite | Tests | Archivos | Runner |
|-------|------:|---------:|--------|
| Backend Rust | **33** | 13 módulos `#[cfg(test)]` | `cargo test` |
| Frontend TypeScript | **155** | 15 archivos `*.test.ts` | `bun test` |
| **Total** | **188** | — | — |

- No hay tests E2E, de navegador real ni de componentes/clicks.
- No hay tests de integración en `src-tauri/tests/`.
- Los tests de frontend cubren funciones puras, clases de servicio y orquestación de comandos con dobles de Tauri.

### 1.1 Cómo ejecutar

```bash
# Backend
cd src-tauri
cargo test

# Frontend
bun test                 # una corrida
bun test --watch         # modo watch
bun test --coverage      # cobertura

# Atajos desde la raíz
bun run test
bun run test:watch
bun run test:coverage
```

Para listar los tests de Rust sin ejecutarlos:

```bash
cd src-tauri
cargo test -- --list
```

### 1.2 Convenciones

**Backend**

- La mayoría de los tests usa SQLite en memoria (`:memory:`) y aplica las migraciones con `db::migrate`.
- Los tests de filesystem, backup y exports usan `tempfile`.
- Los tests asíncronos se ejecutan con `tauri::async_runtime::block_on`.
- Los tests de dominio crean datos reales a través de los `service`, salvo casos puntuales donde se insertan filas para simular estados.

**Frontend**

- Los tests viven junto al código, con sufijo `.test.ts`.
- Se usa `bun:test` (`describe`, `test`, `expect`, `mock`, `spyOn`, `setSystemTime`) sin globals.
- Los módulos con runes (`*.svelte.ts`) se compilan con `scripts/svelte-module-loader.ts`, registrado como `preload` en `bunfig.toml`. El loader quita los tipos con `typescript` y aplica `svelte/compiler` (`compileModule`), así los tests pueden importar clases que usan `$state` y `$derived`.
- No se usa jsdom ni ningún DOM de navegador. Cuando un test necesita `window`, `document` o `localStorage`, se instalan dobles mínimos y se restauran en `afterEach`.
- Los comandos Tauri se reemplazan por dobles compartidos en `src/lib/testing/tauriMocks.ts`; los factories de DTOs viven en `src/lib/testing/factories.ts`.
- Las fechas dependientes de “hoy” usan `setSystemTime` y se restauran al finalizar.

### 1.3 Distribución del backend

| Módulo | Archivo | Tests |
|--------|---------|------:|
| Base de datos | `src-tauri/src/db.rs` | 4 |
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
| **Total** | | **33** |

### 1.4 Distribución del frontend

| Área | Archivos |
|------|----------|
| Utilidades | `src/lib/utils/format.test.ts` |
| API y errores | `src/lib/api/{errors,labels,invoke}.test.ts` |
| Servicios de dominio | `src/lib/services/{staffService,staffServiceCrud,placesService,cleaningService,incidentsService}.test.ts` |
| Reportes y exports | `src/lib/services/{reportsService,exportService}.test.ts` |
| Ajustes, tema y respaldo | `src/lib/services/{settingsService,themeService,backupService}.test.ts` |
| Fecha | `src/lib/services/dateService.test.ts` |

---

## 2. Infraestructura (backend)

### 2.1 `src-tauri/src/db.rs` — 4 tests

| Test | Qué verifica |
|------|--------------|
| `connect_creates_parent_directories` | `connect` crea los directorios padre y el archivo de BD; las migraciones corren correctamente. |
| `migrations_create_the_expected_schema` | Existen las 9 tablas esperadas (`staff`, `place`, `cleaning_service`, `service_staff`, `incident`, `absence`, `staff_place`, `staff_rest_day`, `settings`), 2 filas iniciales en `settings` y el trigger `trg_service_staff_keep_last`. |
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

## 3. Dominio (backend)

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

## 4. Backup y restauración (backend)

### 4.1 `src-tauri/src/backup.rs` — 2 tests

| Test | Qué verifica |
|------|--------------|
| `backup_and_restore_round_trip_preserves_database_and_photos` | Crea datos (personal, lugar y foto), genera un ZIP de respaldo, modifica la BD viva y borra la foto, y luego restaura: el manifiesto reporta los conteos originales, la BD vuelve a tener solo el lugar original y la foto se recupera con su contenido intacto. |
| `restore_rejects_corrupt_zip_and_keeps_live_database_intact` | Rechaza un ZIP corrupto y un ZIP válido cuyo `housekeeping.db` no es SQLite, ambos con código `invalid_backup`; la BD viva conserva sus datos. |

---

## 5. Frontend

### 5.1 Configuración del runner

| Archivo | Función |
|---------|---------|
| `bunfig.toml` | Registra `scripts/svelte-module-loader.ts` como `preload` de `bun test`. |
| `scripts/svelte-module-loader.ts` | Compila los `*.svelte.ts` con runes para que puedan importarse desde los tests. |
| `src/lib/testing/tauriMocks.ts` | Dobles compartidos de `invoke`, `save` y `open`. |
| `src/lib/testing/factories.ts` | Constructores tipados de DTOs (`makeStaff`, `makePlace`, `makeIncident`, etc.). |

Comandos:

```bash
bun test
bun test --watch
bun test --coverage
bun run test
```

### 5.2 Utilidades y API

**`src/lib/utils/format.test.ts`**

- `parseIsoDate`: fechas válidas, formatos malformados y fechas de calendario inexistentes (`2026-02-30`, `2026-13-01`, `2026-04-31`).
- `todayIso`, `currentMonthIso`, `currentMonthRangeIso`, `nowTimeIso`: padding, fin de mes, febrero bisiesto y diciembre.
- `shiftIsoDate`: cruce de mes/año, días bisiestos y entradas inválidas sin cambios.
- `weekdayFromIsoDate`: lunes = 0 … domingo = 6, fecha inválida = `null`.
- `durationMinutesBetween`: rangos válidos, extremos faltantes, horarios inválidos, fin igual o anterior al inicio.
- `formatLongDate`, `formatDate`, `formatMonth`, `formatTime`: valores válidos, `null`, vacíos, inválidos y fallback personalizado.
- `formatNumber`: separadores `es-UY`.
- `formatDurationMinutes`: minutos, horas exactas, combinaciones, redondeo total y valores no finitos/negativos.
- `capitalize`: texto vacío y primera letra.

**`src/lib/api/errors.test.ts`**

- `AppError`: nombre, código y mensaje.
- `toAppError`: instancia existente, payload `{ code, message }`, mensajes vacíos o con espacios, códigos desconocidos, strings, `null`, `undefined` y valores raros.
- `errorMessage`: mensaje normalizado.

**`src/lib/api/labels.test.ts`**

- `labelOf`: valor conocido, `null`, `undefined`, clave inexistente y fallback.
- `toOptions`: orden y forma `{ value, label }`.
- `weekdayLabels`: siete días, de lunes a domingo.
- Completitud de etiquetas para todos los unions de `types.ts`.

**`src/lib/api/invoke.test.ts`**

- `isTauriRuntime`: sin `window`, sin `__TAURI_INTERNALS__` y con él.
- `call`: comando y argumentos, retorno, normalización de errores payload y string.

### 5.3 Clases de servicio

**`src/lib/services/dateService.test.ts`**

- Fecha inicial, `isToday`, `formattedSelectedDate`.
- `setDate` con espacios y valores vacíos.
- `previousDay`/`nextDay` en meses, años y días bisiestos.
- `goToToday`.

**`src/lib/services/staffService.test.ts`**

- `isWithinSeasonOn`: no temporal, temporal sin fechas, límites inclusivos y fuera de temporada.
- `isAvailableOn`: inactivo, temporada, día de descanso, ausencias con límites inclusivos y ausencias de otra persona.
- `absencesFor` y `availableOn`.

**`src/lib/services/staffServiceCrud.test.ts`**

- `create` con y sin días de descanso/lugares principales; recarga y errores.
- `update` con reemplazo de descansos y lugares.
- `deactivate` con `warning`, `reactivate` y errores bloqueados.
- Altas, ediciones y bajas de ausencias, con refresco de la lista y errores.

**`src/lib/services/placesService.test.ts`**

- `load` con fecha, reutilización de `referenceDate` y `null`.
- Creación, edición, `setStatusOverride`, baja y reactivación.
- `recommendedTask` con fecha por defecto.
- Errores traducidos y flags de carga/guardado.

**`src/lib/services/cleaningService.test.ts`**

- `loadByDate` con fecha por defecto y errores.
- `create` con `warning` y recarga.
- `update` con `confirmReopen`.
- `setStatus` usando horas existentes o nuevas.
- `setStaff` con `warning`.
- `suggestStaff`: horas vacías a `null`, `count` por defecto, valores explícitos y limpieza de sugerencias ante error.

**`src/lib/services/incidentsService.test.ts`**

- `openItems` filtra incidencias abiertas.
- `load` guarda y reutiliza filtros.
- `create` y `setStatus` recargan la lista.
- Errores traducidos.

**`src/lib/services/reportsService.test.ts`**

- Totales derivados: servicios, horas con redondeo a dos decimales, servicios completados y días de ausencia; colecciones vacías.
- Loaders de servicios por persona, tiempos promedio, ambos reportes de rango, ausencias por mes y planilla diaria.
- Errores traducidos.

**`src/lib/services/settingsService.test.ts`**

- `load` exitoso y con error.
- `save` exitoso y con error.
- Los defaults exportados no se mutan.

**`src/lib/services/themeService.test.ts`**

- `init` con tema claro, oscuro, valor inválido y una sola ejecución.
- `toggle` y `set` actualizan `data-theme` y `localStorage`.
- Fallos de lectura/escritura de storage.
- Funcionamiento sin DOM.

**`src/lib/services/exportService.test.ts`**

- `suggestedFileName` para planilla diaria, reportes de rango y ausencias por mes, con fallbacks a la fecha/mes actual.
- Exportación fuera de Tauri.
- Cancelación del diálogo.
- Mapeo de formatos a `export_csv`, `export_xlsx` y `export_pdf`.
- Errores de diálogo y de `invoke`; limpieza de estado y `clearError`.

**`src/lib/services/backupService.test.ts`**

- Creación de respaldo fuera de Tauri, cancelada, exitosa y con errores.
- `pickRestoreFile` con archivo, cancelación, selección vacía y errores.
- Restauración exitosa con refresco de los servicios cacheados.
- Error de restauración, `busy` y `clearError`.

### 5.4 Bugs corregidos junto con los tests

1. **`parseIsoDate` aceptaba fechas inexistentes** por el rollover de `Date` (`2026-02-30` → 2 de marzo). Ahora valida año, mes y día contra el `Date` construido.
2. **`formatDurationMinutes` mostraba `"60 min"`** para valores como `59.6`. Ahora redondea el total antes de separar horas y minutos.
3. **`formatTime` y `durationMinutesBetween` aceptaban horas inválidas** (`99:99`, `24:00`, `09:60`). Ahora comparten un parser que valida los rangos.
4. **`shiftIsoDate` saltaba silenciosamente a “hoy”** con una fecha inválida. Ahora devuelve el valor original.
5. Se exportaron las clases `SettingsService` y `ThemeService` para poder instanciarlas aisladas en los tests.

---

## 6. Cobertura y huecos

### 6.1 Bien cubierto

- Reglas de negocio del backend: disponibilidad, recomendaciones, validaciones, transiciones de estado, bloqueos por integridad, unicidad y solapamientos.
- Migraciones, esquema, trigger de integridad y recuperación ante BD corrupta.
- Backup/restauración completo (datos + fotos) y rechazo de respaldos inválidos.
- Cálculo de métricas de reportes y generación de archivos CSV/XLSX/PDF.
- Utilidades de formato, normalización de errores, etiquetas y wrapper de `invoke`.
- Lógica de disponibilidad, navegación de fechas, totales de reportes y temas.
- Orquestación de servicios: comandos, argumentos, recarga, warnings, errores y flags.

### 6.2 Sin cobertura (por decisión)

- **Componentes Svelte:** renderizado, eventos, formularios, navegación y estilos.
- **Tests E2E:** no se usan Playwright, navegador real ni jsdom.
- **Capa IPC real:** los comandos Tauri se mockean; el contrato real está cubierto indirectamente por los tests Rust.
- **Contenido exacto de los exports:** los tests Rust validan formato, firma y tamaño, no cada celda o fila.
- **Flujos visuales:** diálogos nativos, toasts, estados vacíos y manejo visual de errores.

### 6.3 Posibles mejoras

1. Añadir tests de componentes solo si en el futuro se decide cubrir la UI (no es el objetivo actual).
2. Probar los comandos Tauri de punta a punta con una base de datos temporal si se agrega una suite de integración.
3. Verificar el contenido de los reportes exportados, no solo su validez formal.
4. Ejecutar `bun test` y `cargo test` en CI para Windows, Linux y macOS.
