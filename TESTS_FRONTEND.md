# TESTS_FRONTEND.md — Tests de frontend

> Documentación de la suite de tests del frontend TypeScript. **Estado:** MVP `0.1.0`. **Fecha:** 2026-10-22.
>
> Este documento describe **qué prueba cada test**. Las reglas de negocio, el modelo de datos y los contratos son fuente de verdad en `PLAN.md`; aquí solo se documenta su verificación automatizada.
>
> La suite de backend está documentada por separado en `TESTS_BACKEND.md`.

---

## 1. Panorama

| Suite | Tests | Archivos | Runner |
|-------|------:|---------:|--------|
| Frontend TypeScript | **155** | 15 archivos `*.test.ts` | `bun test` |

- No hay tests E2E, de navegador real, de componentes ni de clicks.
- No se usa Playwright, jsdom ni ninguna librería de DOM.
- Se testean funciones puras, clases de servicio y orquestación de comandos con dobles de Tauri.
- Los tests viven junto al código, con sufijo `.test.ts`.

### 1.1 Cómo ejecutar

```bash
bun test                 # una corrida
bun test --watch         # modo watch
bun test --coverage      # cobertura

# Atajos desde la raíz
bun run test
bun run test:watch
bun run test:coverage
```

Para ejecutar un archivo puntual:

```bash
bun test src/lib/utils/format.test.ts
```

### 1.2 Distribución

| Área | Archivo | Tests |
|------|---------|------:|
| Utilidades | `src/lib/utils/format.test.ts` | 27 |
| API y errores | `src/lib/api/errors.test.ts` | 10 |
| API y errores | `src/lib/api/labels.test.ts` | 6 |
| API y errores | `src/lib/api/invoke.test.ts` | 7 |
| Fecha | `src/lib/services/dateService.test.ts` | 8 |
| Personal | `src/lib/services/staffService.test.ts` | 10 |
| Personal | `src/lib/services/staffServiceCrud.test.ts` | 11 |
| Lugares | `src/lib/services/placesService.test.ts` | 11 |
| Limpieza | `src/lib/services/cleaningService.test.ts` | 13 |
| Incidencias | `src/lib/services/incidentsService.test.ts` | 9 |
| Reportes | `src/lib/services/reportsService.test.ts` | 10 |
| Ajustes | `src/lib/services/settingsService.test.ts` | 5 |
| Tema | `src/lib/services/themeService.test.ts` | 9 |
| Exportaciones | `src/lib/services/exportService.test.ts` | 12 |
| Respaldos | `src/lib/services/backupService.test.ts` | 13 |
| **Total** | | **161** |

---

## 2. Infraestructura del runner

| Archivo | Función |
|---------|---------|
| `bunfig.toml` | Registra `scripts/svelte-module-loader.ts` como `preload` de `bun test`. |
| `scripts/svelte-module-loader.ts` | Compila los `*.svelte.ts` con runes para que puedan importarse desde los tests. |
| `src/lib/testing/tauriMocks.ts` | Dobles compartidos de `invoke`, `save` y `open`. |
| `src/lib/testing/factories.ts` | Constructores tipados de DTOs (`makeStaff`, `makePlace`, `makeIncident`, etc.). |

### 2.1 Loader de módulos Svelte

Bun no interpreta las runes (`$state`, `$derived`) por sí solo. El loader:

1. Intercepta archivos `*.svelte.ts` y `*.svelte.js`.
2. Ignora `node_modules`.
3. Quita los tipos con `typescript` (`ts.transpileModule`).
4. Compila las runes con `svelte/compiler` (`compileModule`).
5. Devuelve JavaScript para que Bun lo ejecute.

Esto permite instanciar las clases reales de los servicios sin mockear toda la capa de estado.

### 2.2 Convenciones

- Se usa `bun:test` (`describe`, `test`, `expect`, `mock`, `spyOn`, `setSystemTime`) sin globals.
- Los comandos Tauri se reemplazan con `invokeMock`; los diálogos con `saveDialogMock` y `openDialogMock`.
- Los mocks de módulos son compartidos porque `mock.module` de Bun es process-global.
- Las fechas dependientes de “hoy” usan `setSystemTime` y se restauran en `afterEach`.
- Cuando un test necesita `window`, `document` o `localStorage`, se instalan dobles mínimos y se restauran en `afterEach`.
- Los errores esperados no ensucian la salida: los tests que prueban fallbacks silencian `console.error` con `spyOn`.

---

## 3. Utilidades

### 3.1 `src/lib/utils/format.test.ts` — 27 tests

| Test | Qué verifica |
|------|--------------|
| `parses a valid ISO date as a local date` | `parseIsoDate('2026-10-21')` devuelve un `Date` local con año, mes y día correctos. |
| `accepts a leap day in a leap year` | `2028-02-29` es una fecha válida. |
| `rejects malformed formats` | Rechaza `21/10/2026`, `2026-1-1` y cadena vacía. |
| `rejects dates that do not exist instead of rolling them over` | Rechaza `2026-02-29`, `2026-02-30`, `2026-04-31`, mes `00` y mes `13`. |
| `todayIso pads month and day` | `todayIso` produce `2026-01-05` y `2026-12-31` con padding correcto. |
| `todayIso uses the current date by default` | Sin argumento usa la fecha actual del sistema simulado. |
| `currentMonthIso pads the month` | `currentMonthIso` produce `2026-01` y `2026-12`. |
| `currentMonthRangeIso returns the first and last day of the month` | Calcula el rango de febrero normal, febrero bisiesto, abril y diciembre. |
| `nowTimeIso pads hours and minutes` | Produce `00:00`, `09:05` y `23:59`. |
| `shiftIsoDate moves across month and year boundaries` | Suma y resta días cruzando fin de mes y fin de año. |
| `shiftIsoDate handles leap days` | `2028-02-28 + 1` → `2028-02-29`; `2028-02-29 + 1` → `2028-03-01`; año no bisiesto → `2026-03-01`. |
| `shiftIsoDate leaves invalid values untouched` | Una fecha inválida se devuelve sin cambios en vez de saltar a “hoy”. |
| `weekdayFromIsoDate returns ISO indexes from Monday to Sunday` | Lunes = 0, miércoles = 2, domingo = 6; fecha inválida = `null`. |
| `computes whole minutes between two valid times` | `09:00`→`10:30` = 90 min; `00:00`→`23:59` = 1439 min. |
| `returns null when an endpoint is missing` | `null`, `undefined` o cadena vacía devuelven `null`. |
| `returns null for invalid or non-positive ranges` | Rechaza formatos sin padding, rangos invertidos, duración cero y horas/minutos fuera de rango. |
| `capitalize only changes the first character` | `octubre` → `Octubre`; cadena vacía → cadena vacía. |
| `formatLongDate formats strings and dates in es-UY` | Formatea `2026-10-21` como `Miércoles, 21 de octubre de 2026`. |
| `formatLongDate falls back for invalid values` | Fecha inválida, `Date` inválido y fallback personalizado. |
| `formatDate handles valid and missing values` | Fecha válida, `null`, `undefined`, vacío e inválido con fallback. |
| `formatMonth handles valid and invalid months` | `2026-10` → `Octubre de 2026`; `2026-13`, `2026-1` y vacío con fallback. |
| `formatTime handles valid and invalid times` | `09:05` y `23:59` válidos; `null`, `9:05`, `24:00`, `09:60` y `99:99` con fallback. |
| `formatNumber uses the es-UY separators` | `0`, `1234.5` → `1.234,5`, `-1234.5` → `-1.234,5`. |
| `formatDurationMinutes formats hours and minutes` | `0`, `45`, `60`, `90` y `125` minutos. |
| `formatDurationMinutes rounds the total before splitting hours` | `59.4` → `59 min`; `59.6` → `1 h`; `119.6` → `2 h`. |
| `formatDurationMinutes rejects invalid values` | Negativos, `NaN` e infinito devuelven `Sin datos`. |

---

## 4. API y errores

### 4.1 `src/lib/api/errors.test.ts` — 10 tests

| Test | Qué verifica |
|------|--------------|
| `keeps the code and message` | `AppError` conserva `name`, `code` y `message`. |
| `returns the same instance when it is already an AppError` | `toAppError` no envuelve dos veces una instancia existente. |
| `maps a command error payload` | `{ code, message }` se convierte en `AppError` con esos valores. |
| `trims payload messages` | Elimina espacios al inicio y al final del mensaje. |
| `uses the code message when the payload message is blank` | Mensaje vacío o solo espacios usa el texto por código (`validation`, `duplicate`). |
| `falls back when the code is unknown and the message is blank` | Código desconocido + mensaje vacío usa el mensaje genérico. |
| `maps string causes to an unexpected error` | Un string se normaliza como `unexpected` conservando el texto. |
| `falls back for blank strings` | String vacío usa el mensaje genérico y registra el error. |
| `falls back for unknown values` | `null`, `undefined`, número, objeto y array usan el mensaje genérico. |
| `returns the normalized message` | `errorMessage` devuelve el mensaje de un payload o de un `AppError`. |

### 4.2 `src/lib/api/labels.test.ts` — 6 tests

| Test | Qué verifica |
|------|--------------|
| `returns the Spanish label for a known value` | `female` → `Mujer`; `in_progress` → `En progreso`. |
| `returns the fallback for null, undefined or unknown values` | `null`, `undefined`, clave inexistente y fallback personalizado. |
| `maps labels to option objects preserving order` | `toOptions` genera `{ value, label }` en el orden del record. |
| `works with an empty record` | Un record vacío produce un array vacío. |
| `starts on Monday and has seven entries` | `weekdayLabels` tiene 7 elementos, de `Lunes` a `Domingo`. |
| `every enum value has a label` | Todos los valores de los unions de `types.ts` tienen etiqueta. |

### 4.3 `src/lib/api/invoke.test.ts` — 7 tests

| Test | Qué verifica |
|------|--------------|
| `returns false without a window` | `isTauriRuntime()` es `false` si no existe `window`. |
| `returns false when the Tauri internals are missing` | Un `window` sin `__TAURI_INTERNALS__` no se considera Tauri. |
| `returns true when the Tauri internals are present` | Con `__TAURI_INTERNALS__` devuelve `true`. |
| `invokes the command and returns its result` | `call` pasa comando y argumentos y devuelve el resultado. |
| `passes undefined arguments through` | Sin argumentos llama a `invoke(command, undefined)`. |
| `normalizes a command error payload` | Un rechazo `{ code, message }` se convierte en `AppError`. |
| `normalizes string errors` | Un rechazo string se convierte en `AppError` con código `unexpected`. |

---

## 5. Servicios

### 5.1 `src/lib/services/dateService.test.ts` — 8 tests

| Test | Qué verifica |
|------|--------------|
| `starts on the current date` | La fecha inicial es la actual; `isToday` es `true` y el texto largo es correcto. |
| `setDate trims the value and ignores blanks` | Recorta espacios, actualiza la fecha e ignora valores vacíos. |
| `previousDay and nextDay move one day at a time` | Navegación básica hacia atrás y adelante. |
| `moves across month and year boundaries` | `2026-12-31 + 1` → `2027-01-01`; `2026-01-01 - 1` → `2025-12-31`. |
| `handles leap days` | `2028-02-28 + 1` → `2028-02-29`; `2028-02-29 + 1` → `2028-03-01`. |
| `goToToday returns to the current date` | Vuelve a la fecha actual y `isToday` queda en `true`. |
| `isToday updates when the selected date changes` | El derivado reacciona al cambiar de día y volver. |
| `formattedSelectedDate follows the selected date` | El texto largo se recalcula para la fecha seleccionada. |

### 5.2 `src/lib/services/staffService.test.ts` — 10 tests

| Test | Qué verifica |
|------|--------------|
| `non-seasonal staff are always within season` | Personal no temporal está dentro de temporada en cualquier fecha. |
| `seasonal staff without dates are treated as within season` | Temporal sin fechas no se bloquea por temporada. |
| `seasonal boundaries are inclusive` | Inicio y fin de temporada cuentan como dentro. |
| `inactive staff are never available` | Personal inactivo no está disponible. |
| `seasonal staff are unavailable outside the season` | Fuera de temporada no está disponible; dentro sí. |
| `rest days block availability by ISO weekday` | Un descanso el miércoles (2) bloquea ese día y no el siguiente. |
| `absences block availability including both boundaries` | Una ausencia bloquea inicio, días intermedios y fin; los días externos quedan libres. |
| `absences from another staff member do not affect availability` | Solo aplican las ausencias del mismo `staff_id`. |
| `absencesFor filters by staff id` | Devuelve solo las ausencias de la persona y `[]` si no hay. |
| `availableOn filters active, seasonal, rest day and absence rules` | La lista final excluye inactivos, fuera de temporada, descanso y ausentes. |

### 5.3 `src/lib/services/staffServiceCrud.test.ts` — 11 tests

| Test | Qué verifica |
|------|--------------|
| `creates staff without optional relations and reloads` | Alta sin descansos ni lugares; no llama a los comandos opcionales y recarga. |
| `sets rest days and main places when provided` | Alta con descansos y lugares ejecuta `staff_rest_days_set` y `staff_places_set`. |
| `returns null and stores the error when creation fails` | Error de alta devuelve `null`, guarda el mensaje y apaga `saving`. |
| `updates staff and replaces rest days and places` | Edición ejecuta `staff_update`, reemplaza descansos y lugares. |
| `deactivate stores the warning returned by the backend` | Baja con advertencia guarda `lastWarning` y recarga. |
| `reactivate updates the staff member` | Reactivación ejecuta el comando y recarga. |
| `deactivate failures return null and store the error` | Baja bloqueada devuelve `null` y guarda el error. |
| `creates an absence and refreshes the absence list` | Alta de ausencia ejecuta el comando y refresca solo ausencias. |
| `updates an absence and refreshes the absence list` | Edición de ausencia ejecuta el comando y refresca. |
| `deletes an absence and refreshes the absence list` | Baja de ausencia devuelve `true` y refresca. |
| `delete failures return false and store the error` | Error al borrar devuelve `false`, guarda el error y apaga `saving`. |

### 5.4 `src/lib/services/placesService.test.ts` — 11 tests

| Test | Qué verifica |
|------|--------------|
| `stores the reference date and passes it to the backend` | `load(date)` guarda `referenceDate`, asigna los lugares y consulta con esa fecha. |
| `reuses the stored reference date when no date is given` | `load()` sin fecha reutiliza la última fecha de referencia. |
| `sends null when there is no reference date` | Sin fecha previa, el backend recibe `{ date: null }`. |
| `stores the translated error when loading fails` | Error de carga se guarda traducido y apaga `loading`. |
| `ignores a stale response that resolves after a newer load` | Una carga vieja que responde tarde no pisa los datos de la carga más reciente. |
| `creates a place and reloads the list` | Alta ejecuta `places_create` y luego recarga. |
| `updates a place and reloads the list` | Edición ejecuta `places_update` y luego recarga. |
| `setStatusOverride maps the place fields into the update input` | El override arma el input con nombre, tipo, ubicación, fechas y notas. |
| `deactivates and reactivates a place` | Ejecuta `places_deactivate` y `places_reactivate`. |
| `returns the recommended task and defaults to today` | `recommendedTask` usa la fecha seleccionada o la actual. |
| `stores errors and returns null when a mutation fails` | Error de mutación devuelve `null`, guarda el mensaje y apaga `saving`. |

### 5.5 `src/lib/services/cleaningService.test.ts` — 13 tests

| Test | Qué verifica |
|------|--------------|
| `defaults to today` | `loadByDate()` sin fecha usa la fecha actual. |
| `stores the translated error when loading fails` | Error de carga se guarda y apaga `loading`. |
| `creates the service, stores its warning and reloads the date` | Alta guarda `warning` y recarga la fecha del servicio. |
| `returns null and stores the error when creation fails` | Error de alta devuelve `null`, guarda el mensaje y apaga `saving`. |
| `passes confirmReopen and stores the warning` | Edición envía `confirmReopen` y conserva la advertencia. |
| `falls back to the service times when no new times are provided` | `setStatus` usa las horas existentes si no se pasan nuevas. |
| `uses provided times over the service times` | `setStatus` prioriza las horas nuevas. |
| `sets the staff, stores the warning and reloads` | `setStaff` guarda la advertencia y recarga. |
| `maps empty times to null and uses the default count` | Sugerencias con horas vacías envían `null` y `count: 10`. |
| `passes explicit times and count` | Sugerencias envían horas y cantidad explícitas. |
| `clears suggestions and stores the error on failure` | Error de sugerencias limpia la lista, guarda el mensaje y apaga `loadingSuggestions`. |

### 5.6 `src/lib/services/incidentsService.test.ts` — 9 tests

| Test | Qué verifica |
|------|--------------|
| `keeps only open incidents` | `openItems` deja solo incidencias con estado `open`. |
| `returns an empty list when there are no incidents` | Sin datos, `openItems` es `[]`. |
| `stores the filters and sends them to the backend` | `load(filters)` guarda y envía los filtros. |
| `reuses the last filters when none are given` | `load()` sin filtros reutiliza los últimos. |
| `stores the translated error when loading fails` | Error de carga se guarda y apaga `loading`. |
| `ignores a stale response that resolves after a newer load` | Una carga vieja que responde tarde no pisa los datos de la carga más reciente. |
| `creates an incident and reloads the list` | Alta ejecuta `incidents_create` y recarga. |
| `sets the status and reloads the list` | Cambio de estado ejecuta `incidents_set_status` y recarga. |
| `returns null and stores the error when a mutation fails` | Error de mutación devuelve `null`, guarda el mensaje y apaga `saving`. |

### 5.7 `src/lib/services/reportsService.test.ts` — 10 tests

| Test | Qué verifica |
|------|--------------|
| `sums services and working hours` | Suma `service_count` y `total_hours` de las filas. |
| `rounds total hours to two decimals` | `0.1 + 0.2` → `0.3`, sin errores de coma flotante. |
| `sums completed services and absence days` | Suma servicios completados y días de ausencia. |
| `returns zero for empty collections` | Todos los totales son `0` sin datos. |
| `loads services by staff for a range` | Consulta `reports_services_by_staff` con el rango y asigna el resultado. |
| `loads average times for a range` | Consulta `reports_average_times` con el rango. |
| `loadRangeReports loads both reports` | Ejecuta ambas consultas de rango y asigna cada resultado. |
| `loads absences by month` | Consulta `reports_absences_by_month` con el mes. |
| `loads the daily sheet` | Consulta `reports_daily_sheet` con la fecha. |
| `stores the translated error when a report fails` | Error de reporte se guarda y apaga `loading`. |

### 5.8 `src/lib/services/settingsService.test.ts` — 5 tests

| Test | Qué verifica |
|------|--------------|
| `loads the settings returned by the backend` | `settings_get` asigna los ajustes y limpia el estado de carga. |
| `stores the translated error when loading fails` | Error de carga se guarda traducido. |
| `saves the input and updates the settings` | `settings_update` recibe el input, actualiza el estado y apaga `saving`. |
| `returns false and stores the error when saving fails` | Error de guardado devuelve `false`, guarda el mensaje y apaga `saving`. |
| `the service does not mutate the exported defaults` | Mutar la instancia no modifica `DEFAULT_SETTINGS` ni nuevas instancias. |

### 5.9 `src/lib/services/themeService.test.ts` — 9 tests

| Test | Qué verifica |
|------|--------------|
| `defaults to light when there is nothing stored` | Sin valor persistido usa `light` y lo aplica al DOM. |
| `reads the persisted dark theme` | Un valor `dark` en `localStorage` se aplica al iniciar. |
| `ignores invalid stored values` | Un valor desconocido cae a `light`. |
| `toggle switches and persists the theme` | Alterna claro/oscuro, actualiza `data-theme` y persiste. |
| `set applies and persists an explicit theme` | `set('dark')` actualiza estado, DOM y storage. |
| `init only runs once` | Una segunda llamada a `init` no relee el storage. |
| `survives storage read failures` | Si `getItem` lanza, inicia en `light` sin romper. |
| `survives storage write failures` | Si `setItem` lanza, el tema se aplica igual. |
| `works without a DOM` | `init` y `set` funcionan sin `window`/`document` (SSR/tests). |

### 5.10 `src/lib/services/exportService.test.ts` — 12 tests

| Test | Qué verifica |
|------|--------------|
| `uses the filter date for the daily sheet` | Nombre sugerido `planilla-diaria-2026-10-21.csv`. |
| `falls back to today when the daily sheet has no date` | Sin fecha usa la fecha actual. |
| `uses the full range for range reports` | Nombres con `desde` y `hasta` para servicios y tiempos promedio. |
| `fills missing range endpoints with today` | Si falta un extremo del rango, usa la fecha actual. |
| `uses the month for absences and falls back to the current month` | Usa el mes indicado o el mes actual. |
| `rejects exports outside the Tauri runtime` | Sin Tauri devuelve `null`, muestra el error y no abre diálogo. |
| `does nothing when the save dialog is cancelled` | Cancelar no llama a `invoke` ni deja error. |
| `exports CSV with the expected command and arguments` | `export_csv` recibe `kind`, `filters`, `destPath`; guarda la ruta final. |
| `maps xlsx and pdf formats to their commands` | `xlsx` → `export_xlsx`; `pdf` → `export_pdf`. |
| `reports dialog errors` | Error del diálogo se traduce y devuelve `null`. |
| `reports export errors and clears the active format` | Error de `invoke` se guarda y limpia `exporting`/`activeFormat`. |
| `clearError resets the message` | `clearError` deja `error` en `null`. |

### 5.11 `src/lib/services/backupService.test.ts` — 13 tests

| Test | Qué verifica |
|------|--------------|
| `rejects backup creation outside the Tauri runtime` | Sin Tauri devuelve `null`, muestra el error y no abre diálogo. |
| `does nothing when the save dialog is cancelled` | Cancelar no llama a `invoke` ni deja error. |
| `creates the backup and stores the saved path` | `backup_create` recibe `destPath` y guarda `lastBackupPath`. |
| `stores dialog and backend errors` | Errores de diálogo y de backend se traducen y apagan `creating`. |
| `rejects picking a file outside the Tauri runtime` | Sin Tauri muestra el error de restauración. |
| `returns the selected file` | El diálogo `open` devuelve la ruta seleccionada. |
| `returns null for cancelled or empty selections` | `null`, cadena vacía y array vacío devuelven `null`. |
| `stores dialog errors` | Error al abrir el diálogo se traduce. |
| `restores the backup and refreshes the cached services` | `backup_restore` recibe `srcPath`; recarga ajustes, personal, lugares, limpieza e incidencias; guarda el manifiesto. |
| `stores the translated error when restoring fails` | Error de restauración devuelve `null`, guarda el mensaje y apaga `restoring`. |
| `busy is true while creating or restoring` | `busy` refleja `creating || restoring`. |
| `clearError resets the message` | `clearError` deja `error` en `null`. |

---

## 6. Cobertura y huecos

### 6.1 Bien cubierto

- Utilidades de formato de fechas, horas, duraciones y números.
- Normalización de errores y etiquetas en español.
- Wrapper de `invoke` y detección del runtime Tauri.
- Navegación de fechas y cálculo de disponibilidad de personal.
- Totales de reportes y sugerencias de nombres de archivo.
- Orquestación de servicios: comandos, argumentos, recargas, warnings, errores y flags.
- Tema claro/oscuro, persistencia y tolerancia a fallos de storage.
- Exportaciones y respaldos: diálogos, cancelación, errores y refresco de estado.

### 6.2 Sin cobertura (por decisión)

- **Componentes Svelte:** renderizado, eventos, formularios, navegación y estilos.
- **Tests E2E:** no se usan Playwright, navegador real ni jsdom.
- **Capa IPC real:** los comandos Tauri se mockean; el contrato real está cubierto indirectamente por los tests de backend.
- **Diálogos nativos:** se testea la orquestación, no la UI del sistema operativo.
- **Flujos visuales:** toasts, estados vacíos y manejo visual de errores.

### 6.3 Posibles mejoras

1. Añadir tests de componentes solo si en el futuro se decide cubrir la UI.
2. Probar los comandos Tauri de punta a punta con una base de datos temporal.
3. Verificar el contenido de los reportes exportados, no solo su validez formal.
4. Ejecutar `bun test` en CI para Windows, Linux y macOS.

---

## 7. Bugs corregidos junto con los tests

1. **`parseIsoDate` aceptaba fechas inexistentes** por el rollover de `Date` (`2026-02-30` → 2 de marzo). Ahora valida año, mes y día contra el `Date` construido.
2. **`formatDurationMinutes` mostraba `"60 min"`** para valores como `59.6`. Ahora redondea el total antes de separar horas y minutos.
3. **`formatTime` y `durationMinutesBetween` aceptaban horas inválidas** (`99:99`, `24:00`, `09:60`). Ahora comparten un parser que valida los rangos.
4. **`shiftIsoDate` saltaba silenciosamente a “hoy”** con una fecha inválida. Ahora devuelve el valor original.
5. Se exportaron las clases `SettingsService` y `ThemeService` para poder instanciarlas aisladas en los tests.
