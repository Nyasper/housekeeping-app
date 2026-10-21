# Housekeeping App — Plan definitivo de implementación

> Documento de referencia única para construir el proyecto. Separa **especificaciones funcionales**, **modelo de datos**, **reglas de negocio**, **especificaciones técnicas**, **arquitectura**, **testing** y **roadmap por fases**. Incluye las decisiones finales adoptadas y los cambios respecto al plan original.
>
> Guía operativa para agentes: **`AGENTS.md`**. Registro de cambios post-MVP: **`CHANGELOG.md`**.

---

## 1. Visión y alcance

**Housekeeping App** gestiona el personal de limpieza y los servicios de limpieza de un edificio. Funciona **offline y en local**, sobre un solo equipo, manejada por **una única persona encargada**.

- **En alcance (MVP):** personal, lugares, servicios de limpieza, asignación asistida, tablero diario, historial/reportes, incidencias, exportación (CSV de datos + XLSX/PDF imprimibles), configuración global y backup/restore.
- **Fuera del alcance (fase posterior):** inventario de insumos, multi-usuario, sincronización/red, multi-edificio.

### 1.1 Convención de idioma

- **Identificadores en inglés:** tablas, columnas, tipos, propiedades, funciones, archivos, rutas y módulos.
- **Interfaz 100 % en español:** todos los textos visibles (etiquetas, botones, mensajes, validaciones, errores, fechas, reportes y planillas) están en español. No se implementa i18n multi-idioma en el MVP.
- **Valores de enum en inglés** (`maid`, `window_cleaner`, `occupied`, …) con su etiqueta en español centralizada en `src/lib/api/labels.ts`.

### 1.2 Principios de arquitectura

- Código **modular y mantenible**, separando responsabilidades por dominio.
- Backend Rust con módulos por dominio (`model`, `repo`, `service`, `commands`) y una capa común de infraestructura (`db`, `error`, `state`, `backup`, `settings`).
- Frontend Svelte 5 con **clases de servicio** (`{name}Service.svelte.ts`) que concentran la lógica y el estado; los componentes solo renderizan.
- Interfaz **moderna, rápida e intuitiva**: claridad visual, bajos clics, feedback inmediato y accesibilidad básica (ver 3.11).
- Reutilizar los patrones probados del proyecto hermano **`tauri-inventory-manager`** (ver `src-tauri/src/inventory.rs`): `DbState` con `RwLock`, `connect()` con PRAGMAs, `CommandError` serializable, `VACUUM INTO` para backups, validación de respaldos, swap del pool al restaurar, `settings` key/value y plugin de instancia única.

---

## 2. Decisiones y supuestos clave (confirmados)

| # | Decisión | Opción elegida |
|---|----------|----------------|
| D1 | Modelo de descansos | **Descanso semanal recurrente** (día fijo por funcionario) + **ausencias puntuales** (certificación médica, licencia, falta) con rango de fechas. |
| D2 | Rol del personal | Solo `maid` (Mucama), `window_cleaner` (Vidriero) y `common_areas` (Áreas comunes). |
| D3 | Exportación | **Todo en Rust:** CSV solo para datos; **XLSX** y **PDF** como planillas imprimibles. Sin impresión del navegador. |
| D4 | Fotos | **Archivo en la carpeta de datos de la app**; la BD guarda la ruta relativa (no BLOB). |
| D5 | Inventario | **Fuera del MVP** (fase posterior). |
| D6 | Foto ausente | **Avatar de stock por defecto según `gender`**: uno para hombre y otro para mujer. |
| D7 | Apartamentos asignados | Cada funcionario puede tener **varios apartamentos principales** (N:M en `staff_place`), que se sugieren primero. También puede ser asignado a lugares **no principales**. |
| D8 | Fechas de apartamento | `check_in_date` / `check_out_date` **opcionales**. `NULL` = dato desconocido; no se agrega un valor `unknown` a columnas de fecha. |
| D9 | Zafral | `is_seasonal` + `season_start_date` / `season_end_date` para registrar la temporada. Si hay fechas, restringen la disponibilidad. |
| D10 | Servicios frontend | Clases (`class`) con runes en archivos `{name}Service.svelte.ts`, no closures. |
| D11 | Idioma de la interfaz | **100 % en español** (textos, mensajes, validaciones y reportes). |
| D12 | Tema | **Modo claro por defecto** y **modo oscuro intercambiable**, con preferencia persistida en `localStorage`. |
| D13 | Configuración global | Tabla `settings` key/value + un **`settingsService.svelte.ts` exclusivo** en el frontend. |
| D14 | Backup/restore | **ZIP** con base SQLite + fotos + `manifest.json`; validación, copia de seguridad previa y swap del pool (patrón del proyecto hermano). |
| D15 | Servicios sin personal | **No se permite** crear un servicio sin al menos un funcionario; tampoco se puede quitar el último. |
| D16 | Estado del apartamento | Se agrega **`cancelled` (Cancelado)** para omitir arbitrariamente la limpieza de ese turno. |
| D17 | Changelog | **`CHANGELOG.md` en inglés** (Keep a Changelog + SemVer) para registrar cambios **posteriores al MVP**. |
| D18 | Commits | **Commit automático después de cada feature o cambio grande**, con mensaje breve en inglés (Conventional Commits). |
| D19 | Experiencia de uso | La interfaz debe ser **moderna, rápida e intuitiva/fácil de usar**: claridad visual, bajos clics, feedback inmediato y accesibilidad básica. |

---

## 3. Especificaciones funcionales

### 3.1 Gestión de Personal

**Objetivo:** registrar, editar y dar de baja al personal de limpieza, controlar sus apartamentos principales, su temporada y sus ausencias.

**Datos de personal:**
- `first_name`, `last_name` *(obligatorios)*
- `birth_date`
- `phone`
- `national_id` (cédula) *(obligatorio y único)*
- `address`
- `photo_path` *(opcional — archivo, ver D4)*
- `gender` (`male` | `female`) — **necesario para elegir el avatar por defecto** (D6)
- `role` (`maid` | `window_cleaner` | `common_areas`) *(D2)*
- `is_seasonal` (zafral) *(D9)*
- `season_start_date` / `season_end_date` *(opcionales; si se cargan, ambos y con `start <= end`)*
- `is_active` / dado de baja *(baja lógica, preserva historial)*
- Apartamentos principales *(relación N:M, ver 3.1.1)*
- `notes` *(opcional)*

**Roles del personal (D2):**

| Código | Etiqueta UI | Uso |
|--------|-------------|-----|
| `maid` | Mucama | Limpieza diaria/general/repaso |
| `window_cleaner` | Vidriero | Servicio de vidrios |
| `common_areas` | Áreas comunes | Personal asignado a áreas comunes |

**Zafral (`is_seasonal`) y temporada (D9):**
- `is_seasonal = true` para funcionarios que solo trabajan en un periodo determinado.
- `season_start_date` y `season_end_date` registran la temporada. Se validan ambos-o-ninguno y `start <= end`.
- Si el funcionario es zafral y tiene fechas cargadas, **solo está disponible dentro de la temporada**.
- Si es zafral sin fechas, el flag es informativo (se muestra como insignia y se puede filtrar).
- Si `is_seasonal = 0`, `season_start_date` y `season_end_date` deben ser `NULL`.
- La temporada se muestra en la ficha y en el tablero.

**Fotos y avatar por defecto (D4 + D6):**
- La foto se guarda en `app_data_dir()/photos/staff/{id}/{uuid}.{ext}` y la BD guarda la ruta relativa.
- Si `photo_path` es `null`, se usa un **avatar de stock**:
  - `gender = male` → `static/avatars/default-male.svg`
  - `gender = female` → `static/avatars/default-female.svg`
- Componente reutilizable `Avatar.svelte` que resuelve `photo ?? default`.
- La BD guarda la ruta **relativa**; los DTO de la API devuelven además la ruta **absoluta/URL resuelta** para `convertFileSrc`.
- Al reemplazar una foto, se usa un nombre único o un sufijo de versión para evitar caché del WebView.

**3.1.1 Apartamentos principales (D7):**
- Un funcionario puede tener **varios apartamentos principales** (relación N:M en `staff_place`).
- Los principales se sugieren como **primeras opciones** al asignar un servicio.
- Un funcionario **también puede ser asignado a lugares no principales**; la asignación no se restringe a su lista.
- `staff_place` solo acepta lugares de tipo `apartment` y activos.
- No hay una segunda categoría de asignación; "no principal" = cualquier lugar fuera de esa lista.

**Control de ausencias:**
1. **Descanso semanal recurrente:** uno o más días fijos de la semana por funcionario (ej. domingo).
2. **Ausencias puntuales** con tipo:
   - `medical_certificate` (Certificación médica)
   - `leave` (Licencia)
   - `absence` (Falta por otro motivo)
   - Cada ausencia tiene `start_date`, `end_date` y `reason` opcional.
   - Se rechazan solapes de ausencias para el mismo funcionario.
   - `start_date <= end_date` obligatorio.

**Criterios de aceptación (resumen):**
- Crear/editar/dar de baja/reactivar personal, validando `national_id` único y campos obligatorios.
- Listar con filtros (nombre, rol, activo, zafral, temporada) y búsqueda.
- Configurar días de descanso y ausencias sin solapes.
- Asignar/quitar apartamentos principales.
- Mostrar avatar de stock cuando no haya foto.

### 3.2 Gestión de Lugares

**Objetivo:** administrar apartamentos y áreas comunes del edificio.

- **Tipos:** `apartment` (Apartamento) | `common_area` (Área común).
- **Operaciones:** crear, editar, desactivar y reactivar (baja lógica).
- **Nombre único** (case-insensitive) entre lugares, para evitar duplicados.
- **Estado del apartamento:**

| Código | Etiqueta UI | Significado |
|--------|-------------|-------------|
| `occupied` | Ocupado | Con huésped. |
| `upcoming_occupancy` | Próxima ocupación | Por ocuparse próximamente. |
| `vacant` | Desocupado | Libre. |
| `recently_vacated` | Desocupado reciente | Recién liberado → dispara sugerencia de limpieza. |
| `cancelled` | Cancelado | **Turno omitido:** no se debe limpiar este apartamento en el turno actual (omisión arbitraria). |

- **Semántica de `status` (D16):**
  - `status = NULL` → **automático** derivado de `check_in_date` / `check_out_date`.
  - `status` con valor → **override manual**.
  - `cancelled` es un override manual para omitir el turno; se muestra como insignia y **no genera recomendación**.
  - El tablero muestra desde cuándo está cancelado (`updated_at`) para facilitar revertirlo.
  - Para áreas comunes, `status` siempre es `NULL` (no aplica).
- **Fechas opcionales (D8):**
  - `check_in_date` (ingreso) y `check_out_date` (egreso).
  - Solo aplican a `apartment`.
  - `NULL` = **dato desconocido / no cargado**. No se usa un valor `unknown` en una columna de fecha. En la UI se muestra “Sin datos”.
  - Se permiten fechas parciales (solo una de las dos).
  - Validación: si ambas existen, `check_in_date <= check_out_date`.
- **Semántica de las fechas:** representan la **estadía actual o la próxima** del apartamento. Si no hay estadía conocida, quedan `NULL`.
- **Baja de lugar:** si tiene servicios futuros (`date >= hoy`, `planned` o `in_progress`), se bloquea hasta cancelarlos o reasignarlos a otro lugar.
- `check_in_date`, `check_out_date` y `status` solo se persisten en `apartment`; en `common_area` deben ser `NULL`.
- Campos extra: `location` (piso/ubicación), `notes`.
- Timestamps: `created_at`, `updated_at`.

**3.2.1 Tarea recomendada según fechas (D8):**

Sea `D` la fecha de referencia (hoy o la fecha seleccionada en el tablero). El orden de prioridad es:

| Prioridad | Condición | Estado sugerido | Tarea recomendada |
|-----------|-----------|-----------------|-------------------|
| 1 | `status = cancelled` (override manual) | `cancelled` | **Ninguna** (turno omitido) |
| 2 | `check_out_date` es `D` o está dentro de los `recent_vacated_days` posteriores | `recently_vacated` | `deep_cleaning` (Limpieza general) |
| 3 | `check_in_date` está dentro de los próximos `upcoming_occupancy_days` y aún no está ocupado | `upcoming_occupancy` | `touch_up` (Repaso) |
| 4 | `check_in_date <= D <= check_out_date` | `occupied` | `daily_cleaning` (Limpieza diaria) |
| 5 | Sin fechas suficientes o fuera de rango | `vacant` | Sin recomendación automática |

- Los días de las ventanas se configuran en `settings` (por defecto **2** cada una).
- Con fechas parciales:
  - Solo `check_out_date`: se aplican las reglas 2 y 5.
  - Solo `check_in_date`: se aplican las reglas 3 y 5.
  - Sin fechas: regla 5 (o el override manual si existe).
- Si `status` tiene un override manual distinto de `cancelled`, se muestra ese estado, pero la **recomendación de tarea sigue calculándose por fechas**.
- `glass` (vidrios) no se deriva de fechas; se agenda manualmente o por periodicidad.
- La recomendación es una **sugerencia**; la encargada puede cambiarla.

### 3.3 Servicios de Limpieza

**Objetivo:** asignar y registrar los servicios de limpieza del día.

- Un **servicio** se asigna a un **lugar** (apartamento o área común).
- **Personal obligatorio (D15):** un servicio **no puede crearse sin al menos un funcionario**. Tampoco se puede quitar el último funcionario de un servicio activo.
- **Personal asignado:** normalmente 1 o 2, pero puede ser más (N a M). Uno puede marcarse como responsable principal (`is_lead`), como máximo uno.
- **Datos del servicio:**
  - `place_id`
  - `date` *(clave del tablero)*
  - `start_time` / `end_time` *(opcionales al planificar; obligatorios al completar)*
  - Personal asignado (1..N)
  - `service_type`
  - `status` (`planned` | `in_progress` | `completed` | `cancelled`)
  - `notes` *(opcional)*
  - `created_at`, `updated_at`

- **Tipos de servicio:**

| Código | Etiqueta UI | Rol sugerido |
|--------|-------------|--------------|
| `glass` | Vidrios | `window_cleaner` |
| `daily_cleaning` | Limpieza diaria | `maid` / `common_areas` |
| `deep_cleaning` | Limpieza general | `maid` / `common_areas` |
| `touch_up` | Repaso | `maid` / `common_areas` |

**Transiciones de estado:**
- `planned → in_progress → completed`
- `planned` o `in_progress → cancelled`
- `completed` es terminal; reabrir requiere confirmación explícita.
- Para pasar a `in_progress`: al menos 1 funcionario y `start_time`.
- Para pasar a `completed`: `start_time` y `end_time` (con `start_time < end_time`), y al menos 1 funcionario.
- Un servicio `cancelled` se excluye de reportes y promedios.

**Criterios de aceptación (resumen):**
- Crear un servicio seleccionando lugar, tipo, fecha y al menos un funcionario.
- Editar servicios `planned`; los `completed` solo con confirmación.
- Asignar/reasignar personal sin dejar el servicio vacío.
- Marcar inicio/fin para calcular duración.
- Cancelar un servicio o marcar el turno del apartamento como `cancelled`.

### 3.4 Asignación asistida

**Objetivo:** sugerir personal adecuado y disponible para un servicio.

**Regla de disponibilidad** — un funcionario está **disponible** en la fecha `D` si:
1. Está **activo** (`is_active`).
2. Si es **zafral con temporada cargada**, `D` está dentro de `[season_start_date, season_end_date]`.
3. `D` **no** cae en un día de descanso semanal suyo.
4. No tiene una ausencia puntual cuyo rango `[start_date, end_date]` incluya a `D`.
5. Para **sugerencias**, no tiene otro servicio con horario solapado ese día. Si los horarios no están cargados, se permite pero se advierte.

**Orden de sugerencia (ranking)** para el lugar `L` y tipo `T`:
1. Compatibilidad de rol con el tipo de servicio (tabla de 3.3).
2. **Apartamento principal**: si `L` está entre los apartamentos principales del funcionario, se sugiere primero. Los demás siguen siendo candidatos válidos.
3. Desempate por rotación/carga (menos servicios recientes primero; luego por `id` para orden determinista).

**Salida:** lista ordenada de candidatos con el motivo (rol, principal, rotación); la encargada confirma manualmente. La app **sugiere**, no asigna sola.

**Advertencia adicional:** si se intenta asignar manualmente a alguien con otro servicio solapado, mostrar aviso; si no hay horas cargadas, avisar por mismo día. No bloquear por defecto.

**Al dar de baja a un funcionario:**
- Si tiene servicios futuros (`date >= hoy`, `planned` o `in_progress`) donde es el **único** asignado, se **bloquea** la baja hasta reasignar o cancelar esos servicios.
- Si tiene más personal asignado, se advierte y se permite la baja, pero el servicio queda marcado para revisión.

### 3.5 Pantalla principal — Tablero diario

Vista tipo tablero: **"qué hay que limpiar hoy y quién lo hace"**.

**Secciones:**
1. **Selector de fecha** (hoy por defecto) y resumen del día.
2. **Pendientes sugeridos:** apartamentos con tarea recomendada y sin servicio creado para esa fecha. Excluye `cancelled`.
3. **Servicios del día:** agrupados por lugar/estado, con personal asignado, horas y acciones rápidas.
4. **Personal:** disponibles, con descanso/ausencia/temporada y asignaciones del día.
5. **Incidencias abiertas:** acceso rápido a las no resueltas.

**Acciones rápidas:** crear servicio (con personal obligatorio), marcar en progreso/completado, cancelar servicio, marcar apartamento como `cancelled`, agregar incidencia.

**Resaltados:** `recently_vacated`, tarea recomendada, `cancelled`, servicios con anomalías (por ejemplo, personal dado de baja).

### 3.6 Historial y Reportes

**Reportes del MVP:**
- **Servicios por persona:** cantidad de servicios y total de horas por funcionario en un rango de fechas. Excluye `cancelled`.
- **Tiempos promedio por tipo de servicio:** promedio de `end_time - start_time` por `service_type`; solo servicios `completed` con ambas horas.
- **Ausencias por mes:** días de ausencia por funcionario y tipo, dentro del mes seleccionado. Los rangos que cruzan meses se reparten por día.
- **Planilla diaria:** datos del tablero de una fecha (lugar, tipo, personal, horas, estado, incidencias).

**Exportación (D3):**
- **CSV** → **solo datos** (reportes tabulares). UTF-8 con BOM para Excel.
- **XLSX** → **planilla imprimible** con formato, bordes, anchos, título, fecha y configuración de página.
- **PDF** → **planilla imprimible** equivalente, con tabla, saltos de página y numeración.
- **Todo se genera en Rust**; no se usa la impresión del navegador.
- Los archivos exportados usan siempre un tema claro de alto contraste.

**Rango de fechas:** por defecto el **mes actual**; el usuario puede cambiarlo antes de exportar.

### 3.7 Incidencias

- Registrar durante o después de una limpieza:
  - `damage` (Daño)
  - `lost_item` (Objeto olvidado)
  - `supply_shortage` (Faltante de insumos)
- Campos: `service_id`, tipo, `description` *(obligatoria)*, foto opcional, estado (`open` | `resolved`), `date`, `resolved_at` opcional, `created_at`.
- Se permite registrar incidencias sobre servicios no cancelados (incluidos completados).
- Foto con el mismo flujo de almacenamiento que las fotos de personal (`photos/incidents/...`).
- Listar con filtros por estado, tipo, fecha y servicio; marcar como resuelta.

### 3.8 Backup / Restore (D14)

Inspirado en `tauri-inventory-manager/src-tauri/src/inventory.rs`.

- **Formato:** archivo **ZIP** con:
  - `housekeeping.db` — copia consistente vía `VACUUM INTO`.
  - `photos/` — fotos de personal e incidencias.
  - `manifest.json` — versión de la app, versión de esquema, fecha, conteos y versión del formato.
- **Backup:** se escribe en un temporal y se renombra al final; si falla, no se destruye un backup anterior.
- **Restore:**
  1. Validar ZIP y `manifest.json`.
  2. Extraer la DB a un temporal y validar cabecera SQLite, tablas/columnas esperadas, `PRAGMA integrity_check` y `PRAGMA foreign_key_check`.
  3. Crear una **copia de seguridad** de la DB actual antes de reemplazar.
  4. Cerrar el pool (`DbState` con `RwLock`), reemplazar DB y fotos, reabrir y ejecutar migraciones.
  5. Si falla la reapertura, restaurar automáticamente la copia de seguridad anterior.
- **Arranque:** `open_with_recovery` aparta una base corrupta (`quarantine_database`) y continúa con una base nueva, dejando un `StartupNotice` para el frontend.
- **Instancia única:** plugin `tauri-plugin-single-instance` para evitar dos procesos escribiendo la misma DB.

### 3.9 Configuración global (D13)

- Tabla `settings` key/value para configuraciones de la app.
- Claves del MVP:
  - `recent_vacated_days` (entero, 0–30; por defecto **2**)
  - `upcoming_occupancy_days` (entero, 0–30; por defecto **2**)
- Se pueden agregar claves nuevas sin migración (upsert).
- El frontend tiene un servicio exclusivo `settingsService.svelte.ts` que carga y guarda la configuración.
- Pantalla **Ajustes**: configuración global + backup/restore.
- Validación en backend de rangos y tipos; valores inválidos caen a los defaults.

### 3.10 Apariencia y tema (D12)

- **Modo claro por defecto** y **modo oscuro intercambiable** desde la interfaz.
- El tema aplica a toda la app: formularios, tablas, tablero, reportes y vistas.
- La preferencia se **persiste en `localStorage`** (clave `housekeeping-theme`).
- Implementación con variables CSS sobre `<html>`: `[data-theme="light"]` y `[data-theme="dark"]`, sin librerías de UI.
- Un script mínimo en `app.html` aplica el tema guardado antes del primer render para evitar parpadeo (FOUC).
- Los archivos exportados (XLSX/PDF) usan siempre un tema claro de alto contraste, independientemente del tema activo.

### 3.11 Experiencia de uso (UI/UX) (D19)

**Objetivo:** una interfaz **moderna, rápida e intuitiva**, que la encargada pueda usar sin curva de aprendizaje y con el mínimo de clics.

**Moderna y consistente**
- Diseño limpio y profesional, con jerarquía visual clara, espaciado generoso y tipografía legible.
- Componentes reutilizables y patrones consistentes en todas las pantallas.
- Tokens CSS para color, tipografía, espaciado, radios y sombras; evitar estilos ad hoc.
- Iconografía consistente y textos en español; evitar el aspecto de template genérico.
- Transiciones sutiles, respetando `prefers-reduced-motion`.

**Rápida**
- Svelte 5 eficiente: evitar trabajo reactivo innecesario y listas costosas.
- Carga local inmediata; sin esperas de red.
- Feedback visual en <100 ms para acciones locales; estados de carga/skeleton cuando una operación tarde.
- Búsquedas con debounce; rutas/paneles pesados con carga diferida si hace falta.
- Bundle pequeño: no agregar dependencias de UI innecesarias.

**Intuitiva y fácil de usar**
- Navegación lateral clara: Tablero, Personal, Lugares, Servicios, Incidencias, Reportes, Ajustes.
- Acción principal visible en cada pantalla; acciones destructivas con confirmación.
- Formularios con defaults sensatos, validación inline en español y mensajes claros.
- Estados vacío/cargando/error bien diseñados, con una acción sugerida.
- Búsqueda y filtros accesibles; no obligar a recordar rutas o códigos.
- Atajos básicos: `Esc` cierra modales, `Enter` envía, navegación por teclado y foco visible.
- Toasts para confirmar acciones y errores.
- El tablero diario debe responder de un vistazo: qué limpiar, quién lo hace y qué está pendiente.

**Accesibilidad**
- Objetivo WCAG 2.2 AA básico: contraste, labels, foco, teclado y tamaño de objetivo.
- No depender solo del color para comunicar estado.

**Criterios de aceptación**
- Una persona nueva puede crear un servicio completo (lugar + tipo + fecha + personal) sin ayuda.
- Las acciones frecuentes están a ≤2 clics desde el tablero.
- No hay pantallas sin estado vacío/cargando/error.
- La app se puede usar completamente con teclado en los flujos principales.
- El cambio de tema no produce parpadeo ni rompe estilos.
- Revisión con las skills `frontend-design` y `accessibility` antes de cerrar la fase.

---

## 4. Modelo de datos (SQLite / SQLx)

IDs: `INTEGER PRIMARY KEY AUTOINCREMENT` (Rust `i64`, TS `number`). Fechas/horas: `TEXT` ISO (`YYYY-MM-DD`, `HH:MM`). Bajas = `is_active = 0` (lógica).

### Tablas

**`staff`** — personal
| campo | tipo | notas |
|-------|------|-------|
| id | INTEGER PK | |
| first_name | TEXT NOT NULL | |
| last_name | TEXT NOT NULL | |
| birth_date | TEXT NULL | |
| phone | TEXT NULL | |
| national_id | TEXT NOT NULL UNIQUE | cédula; único incluso dado de baja (reactivar) |
| address | TEXT NULL | |
| photo_path | TEXT NULL | ruta relativa a `photos/staff/...` |
| gender | TEXT NOT NULL | `male` \| `female` (avatar por defecto) |
| role | TEXT NOT NULL | `maid` \| `window_cleaner` \| `common_areas` |
| is_seasonal | INTEGER NOT NULL DEFAULT 0 | zafral |
| season_start_date | TEXT NULL | inicio de temporada |
| season_end_date | TEXT NULL | fin de temporada |
| is_active | INTEGER NOT NULL DEFAULT 1 | baja lógica |
| notes | TEXT NULL | |
| created_at / updated_at | TEXT NOT NULL DEFAULT (datetime('now')) | |

**`staff_rest_day`** — descanso semanal recurrente
| campo | tipo | notas |
|-------|------|-------|
| staff_id | INTEGER NOT NULL FK | |
| weekday | INTEGER NOT NULL | 0=lunes … 6=domingo (ISO, `chrono::Weekday`) |
| PK(staff_id, weekday) | | |

**`absence`** — ausencia puntual
| campo | tipo | notas |
|-------|------|-------|
| id | INTEGER PK | |
| staff_id | INTEGER NOT NULL FK | |
| type | TEXT NOT NULL | `medical_certificate` \| `leave` \| `absence` |
| start_date | TEXT NOT NULL | |
| end_date | TEXT NOT NULL | |
| reason | TEXT NULL | |
| created_at | TEXT NOT NULL DEFAULT (datetime('now')) | |

**`place`** — apartamento / área común
| campo | tipo | notas |
|-------|------|-------|
| id | INTEGER PK | |
| name | TEXT NOT NULL | único case-insensitive |
| type | TEXT NOT NULL | `apartment` \| `common_area` |
| status | TEXT NULL | `occupied` \| `upcoming_occupancy` \| `vacant` \| `recently_vacated` \| `cancelled`; **NULL = automático** |
| location | TEXT NULL | piso/ubicación |
| check_in_date | TEXT NULL | solo apartamentos; NULL = desconocido |
| check_out_date | TEXT NULL | solo apartamentos; NULL = desconocido |
| is_active | INTEGER NOT NULL DEFAULT 1 | |
| notes | TEXT NULL | |
| created_at / updated_at | TEXT NOT NULL DEFAULT (datetime('now')) | |

**`staff_place`** — apartamentos principales asignados (N:M)
| campo | tipo | notas |
|-------|------|-------|
| staff_id | INTEGER NOT NULL FK | |
| place_id | INTEGER NOT NULL FK | solo `apartment` activo |
| PK(staff_id, place_id) | | |

**`cleaning_service`** — servicio de limpieza
| campo | tipo | notas |
|-------|------|-------|
| id | INTEGER PK | |
| place_id | INTEGER FK NOT NULL | |
| date | TEXT NOT NULL | |
| start_time | TEXT NULL | obligatoria al iniciar/completar |
| end_time | TEXT NULL | obligatoria al completar |
| service_type | TEXT NOT NULL | `glass` \| `daily_cleaning` \| `deep_cleaning` \| `touch_up` |
| status | TEXT NOT NULL DEFAULT 'planned' | `planned` \| `in_progress` \| `completed` \| `cancelled` |
| notes | TEXT NULL | |
| created_at / updated_at | TEXT NOT NULL DEFAULT (datetime('now')) | |

**`service_staff`** — personal asignado al servicio (N:M)
| campo | tipo | notas |
|-------|------|-------|
| service_id | INTEGER NOT NULL FK | |
| staff_id | INTEGER NOT NULL FK | |
| is_lead | INTEGER NOT NULL DEFAULT 0 | responsable principal (máx. 1 por servicio) |
| PK(service_id, staff_id) | | |

**`incident`**
| campo | tipo | notas |
|-------|------|-------|
| id | INTEGER PK | |
| service_id | INTEGER FK NOT NULL | |
| type | TEXT NOT NULL | `damage` \| `lost_item` \| `supply_shortage` |
| description | TEXT NOT NULL | |
| photo_path | TEXT NULL | `photos/incidents/...` |
| status | TEXT NOT NULL DEFAULT 'open' | `open` \| `resolved` |
| date | TEXT NOT NULL | |
| resolved_at | TEXT NULL | |
| created_at | TEXT NOT NULL DEFAULT (datetime('now')) | |

**`settings`** — configuración global
| campo | tipo | notas |
|-------|------|-------|
| key | TEXT PK NOT NULL | |
| value | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL DEFAULT (datetime('now')) | |

**Seed de settings:**
- `recent_vacated_days = '2'`
- `upcoming_occupancy_days = '2'`

### Índices y constraints

- `CREATE UNIQUE INDEX ... ON place(name COLLATE NOCASE);`
- `CREATE UNIQUE INDEX ... ON service_staff(service_id) WHERE is_lead = 1;`
- `CREATE INDEX ... ON cleaning_service(date);`
- `CREATE INDEX ... ON cleaning_service(place_id, date);`
- `CREATE INDEX ... ON service_staff(staff_id);`
- `CREATE INDEX ... ON absence(staff_id, start_date, end_date);`
- `CREATE INDEX ... ON incident(service_id);`
- `CREATE INDEX ... ON incident(status);`
- `CREATE INDEX ... ON staff_place(place_id);`
- `CHECK (weekday BETWEEN 0 AND 6)` en `staff_rest_day`.
- `CHECK (start_date <= end_date)` en `absence`.
- `CHECK (check_in_date IS NULL OR check_out_date IS NULL OR check_in_date <= check_out_date)` en `place`.
- `CHECK (start_time IS NULL OR end_time IS NULL OR start_time < end_time)` en `cleaning_service`.
- `CHECK (is_seasonal = 0 AND season_start_date IS NULL AND season_end_date IS NULL) OR (is_seasonal = 1 AND ((season_start_date IS NULL AND season_end_date IS NULL) OR (season_start_date IS NOT NULL AND season_end_date IS NOT NULL AND season_start_date <= season_end_date)))`.
- `CHECK (status IS NULL OR status IN ('occupied','upcoming_occupancy','vacant','recently_vacated','cancelled'))`.
- `CHECK (type IN ('apartment','common_area'))` en `place`; si `type = 'common_area'`, entonces `status`, `check_in_date` y `check_out_date` deben ser `NULL`.
- CHECKs de enums en `staff.role`, `staff.gender`, `absence.type`, `cleaning_service.service_type`, `cleaning_service.status`, `incident.type` e `incident.status`.
- Trigger para impedir eliminar el último `service_staff` de un servicio (defensa adicional a la validación de servicio).

---

## 5. Reglas de negocio (resumen formal)

- **Disponibilidad** = `is_active` ∧ temporada (si zafral con fechas) ∧ (no descanso semanal en D) ∧ (no ausencia que cubra D).
- **Sugerencias** = disponibilidad ∧ sin servicio solapado (si hay horas; si no, advertencia).
- **Ranking** = rol compatible → apartamento principal → rotación/carga → `id` (determinista).
- **Tarea recomendada** = orden de 3.2.1; `cancelled` no genera recomendación.
- **`place.status = NULL`** = automático desde fechas; valor = override manual.
- **`cancelled`** = turno omitido; se muestra y se revierte manualmente.
- **Servicio sin personal:** prohibido. No se puede crear vacío ni quitar el último funcionario.
- **Baja de personal/lugar:** lógica; no se borran físicamente. Al dar de baja con servicios futuros, aplicar reglas de 3.4/3.2.
- **`national_id` único:** si se da de baja, se reactiva; no se re-crea.
- **`place.name` único** (case-insensitive).
- **`staff_place`:** solo apartamentos activos.
- **`is_lead`:** máximo uno por servicio.
- **Ausencias:** sin solapes para el mismo funcionario; fechas coherentes.
- **Servicios:** `start_time < end_time`; completar exige horas y personal; `cancelled` se excluye de reportes.
- **Doble asignación:** aviso por solape; si faltan horas, aviso por mismo día.
- **Ventanas de recomendación:** configurables en `settings`.

---

## 6. Especificaciones técnicas

### 6.1 Stack

- **Lenguaje:** Rust (backend) + TypeScript/Svelte (frontend).
- **Gestor de paquetes:** Bun.
- **Backend:** Tauri v2 (Rust).
- **Frontend:** Svelte 5 + SvelteKit 3, adaptador estático (SPA, `ssr = false`).
- **Estilos:** CSS nativo con variables CSS (design tokens).
- **Base de datos:** SQLite vía SQLx, con migraciones versionadas (`sqlx::migrate!`).
- **Exportación:** Rust (`csv`, `rust_xlsxwriter`, `printpdf`).
- **Backup:** ZIP (`zip`) + SQLite `VACUUM INTO`.

### 6.2 Backend (Rust)

**Dependencias (`src-tauri/Cargo.toml`):**

| crate | versión | uso |
|-------|---------|-----|
| `sqlx` | `0.8` (`runtime-tokio`, `sqlite`, `migrate`, `chrono`) | DB y migraciones |
| `chrono` | `0.4` (`serde`) | fechas/horas |
| `tauri-plugin-dialog` | `2` | diálogos de archivo |
| `tauri-plugin-fs` | `2` | permisos de archivos requeridos |
| `tauri-plugin-single-instance` | `2` | instancia única |
| `csv` | `1.4` | exportación CSV |
| `rust_xlsxwriter` | `0.101` | exportación XLSX imprimible |
| `printpdf` | `0.12` | exportación PDF imprimible |
| `zip` | `9` | backup/restore |
| `walkdir` | `2.5` | recorrer fotos |
| `serde` / `serde_json` | `1` | serialización y manifest |
| Dev: `tempfile` | `3` | tests con archivos temporales |
| Dev: `tokio` | `1` (`macros`, `rt-multi-thread`) | tests async |

> **Justificación de librerías:** `rust_xlsxwriter` es el escritor XLSX más activo y popular del ecosistema (actualización reciente y ~2,4 M descargas recientes); `printpdf` es la librería de creación de PDF de alto nivel más usada y activa (~1,6 M descargas recientes); `csv` es el estándar de CSV (~58 M descargas recientes). Alternativa para PDF con maquetación más rica: `typst`, pero se descarta por peso de compilación para este MVP.

**Patrones adoptados del proyecto hermano (`tauri-inventory-manager/src-tauri/src/inventory.rs`):**

- **`DbState`:** `DbState(RwLock<SqlitePool>)` con `pool()`; permite cerrar y reabrir el pool al restaurar.
- **`connect(path)`:** crea el directorio padre; `SqlitePoolOptions::new().max_connections(1)` para serializar escrituras; `SqliteConnectOptions` con `create_if_missing`, `foreign_keys(true)`, `busy_timeout(10s)`, `journal_mode(Wal)`, `synchronous(Normal)`, `temp_store=2`, `cache_size=-8000`.
- **`CommandError`:** `{ code: &'static str, message: String }`, serializable; mensajes en español. Mapear errores SQLx (unique, foreign key) a códigos claros.
- **`db_path(app)`:** `app_data_dir()/housekeeping.db`; fotos en `app_data_dir()/photos/`.
- **`open_with_recovery`:** abre la DB; si falla por corrupción, la aparta con `quarantine_database`, crea una nueva y deja un `StartupNotice` consumible con `take_startup_notice`.
- **Backup/restore:** `VACUUM INTO` + validación + copia de seguridad + swap del pool (ver 6.6).
- **Settings:** tabla key/value con `get_settings` tipado y `save_settings` con upsert.
- **Instancia única:** `tauri-plugin-single-instance` enfoca la ventana existente.

**Estructura de módulos (todo en inglés):**
```
src-tauri/
  assets/
    fonts/
      DejaVuSans.ttf      # fuente embebida para PDF (acentos/ñ)
  migrations/
    0001_initial.sql
  src/
    main.rs
    lib.rs                  # Builder, plugins, setup, comandos
    error.rs                # CommandError + mapeo SQLx
    db.rs                   # connect, open_with_recovery, db_path, quarantine
    state.rs                # DbState(RwLock<Pool<Sqlite>>), StartupNotice
    settings.rs             # get_settings / save_settings (tipados)
    backup.rs               # ZIP + validación + restore + swap de pool
    domain/
      staff/
        mod.rs
        model.rs
        repo.rs
        service.rs
        commands.rs
      places/  (idem)
      cleaning/ (idem — servicios de limpieza)
      incidents/ (idem)
      reports/
        mod.rs
        queries.rs
        export_csv.rs
        export_xlsx.rs
        export_pdf.rs
        commands.rs
    util/
      dates.rs              # fechas, temporadas, ventanas
      availability.rs       # disponibilidad + ranking + recomendación
      files.rs              # fotos, rutas, validación de imágenes
```

> **Nota:** el proyecto hermano concentra la lógica en `inventory.rs` + `queries.rs`. Aquí se mantienen esos patrones, pero con separación por dominio para cumplir el requisito de modularidad.

**Migraciones:** `sqlx::migrate!("./migrations").run(&pool)` en `setup`. La migración inicial crea todas las tablas, índices, constraints, triggers y el seed de `settings`.

### 6.3 Frontend (Svelte 5 / SvelteKit 3)

**Estructura (archivos y rutas en inglés):**
```
src/
  app.html                     # lang="es", script de tema (FOUC)
  routes/
    +layout.svelte             # AppShell, navegación, toggle de tema
    +layout.ts
    +page.svelte               # Tablero diario
    staff/                     # listado + formulario + ausencias
    places/
    cleaning/
    incidents/
    reports/
    settings/                  # configuración global + backup/restore
  lib/
    api/
      types.ts                 # DTOs TS (espejo de Rust, en inglés)
      invoke.ts                # wrappers tipados de invoke
      errors.ts                # CommandError -> mensaje español
      labels.ts                # códigos de enum -> etiquetas español
    utils/
      format.ts                # formato de fechas/números en es-UY
    services/                  # lógica separada de UI (clases)
      staffService.svelte.ts
      placesService.svelte.ts
      cleaningService.svelte.ts
      incidentsService.svelte.ts
      reportsService.svelte.ts
      exportService.svelte.ts
      backupService.svelte.ts
      settingsService.svelte.ts  # exclusivo para la tabla settings
      themeService.svelte.ts
      dateService.svelte.ts      # fecha activa del tablero
    components/
      AppShell.svelte
      Sidebar.svelte
      ThemeToggle.svelte
      Avatar.svelte
      StaffForm.svelte
      PlaceForm.svelte
      ServiceForm.svelte
      DailyBoard.svelte
      AvailabilityList.svelte
      IncidentForm.svelte
      ConfirmDialog.svelte
      Toast.svelte
      EmptyState.svelte
      FormField.svelte
    styles/
      tokens.css               # variables para tema claro/oscuro
static/
  avatars/
    default-male.svg
    default-female.svg
```

**Patrón de servicios (clases, no closures):** la lógica vive fuera de los componentes, en `{name}Service.svelte.ts`, con una **clase** cuyos campos usan runes (`$state`, `$derived`). Los componentes solo renderizan y llaman métodos del servicio.

```ts
// src/lib/services/staffService.svelte.ts
import { invoke } from '@tauri-apps/api/core';
import type { Staff, NewStaff } from '#lib/api/types.js';

export class StaffService {
  items = $state<Staff[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);

  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.items = await invoke<Staff[]>('staff_list');
    } catch (cause) {
      this.error = String(cause);
    } finally {
      this.loading = false;
    }
  }

  async create(input: NewStaff): Promise<void> {
    await invoke('staff_create', { input });
    await this.load();
  }

  availableOn(date: string): Staff[] {
    return this.items.filter((staff) => staff.is_active /* + descansos/ausencias/temporada */);
  }
}

export const staffService = new StaffService();
```

**Servicio exclusivo de configuración:**

```ts
// src/lib/services/settingsService.svelte.ts
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, SettingsInput } from '#lib/api/types.js';

export const DEFAULT_SETTINGS: AppSettings = {
  recent_vacated_days: 2,
  upcoming_occupancy_days: 2
};

export class SettingsService {
  settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });
  loading = $state(false);

  async load(): Promise<void> {
    this.settings = await invoke<AppSettings>('settings_get');
  }

  async save(input: SettingsInput): Promise<void> {
    this.settings = await invoke<AppSettings>('settings_update', { input });
  }
}

export const settingsService = new SettingsService();
```

**Convenciones:**
- Svelte 5 moderno: runes (`$state`, `$derived`, `$effect`, `$props`, `$bindable`), snippets (`{#snippet}` / `{@render}`), attachments (`{@attach}`).
- Usar **skills y MCP de Svelte** al crear/editar archivos `.svelte`.
- CSS nativo con variables y estilos scoped.
- **Tema:** `tokens.css` define variables para `[data-theme="light"]` (default) y `[data-theme="dark"]`; `themeService` persiste en `localStorage` y aplica el atributo en `<html>`.
- **Idioma:** todos los textos visibles en español; `labels.ts` centraliza etiquetas de enums.
- **Formato de fechas:** `Intl.DateTimeFormat('es-UY', ...)`; helper en `dateService`/`format.ts`.
- **Avatar por defecto:** `Avatar.svelte` recibe `photoPath`, `gender`, `alt` y resuelve:
  ```ts
  const fallback = gender === 'female'
    ? asset('avatars/default-female.svg')
    : asset('avatars/default-male.svg');
  ```
- **Errores:** `errors.ts` traduce `{ code, message }` a mensajes en español; los componentes muestran el mensaje.
- **UI/UX:** diseño moderno, consistente y de bajos clics; usar tokens; implementar estados vacío/cargando/error; validación inline; confirmaciones destructivas; foco/teclado; `prefers-reduced-motion`.
- **Rendimiento:** evitar trabajo reactivo innecesario, debounce en búsqueda y carga diferida de paneles pesados; mantener el bundle pequeño.
- **Skills:** usar `frontend-design` y `accessibility` al diseñar o revisar UI.

### 6.4 API de comandos Tauri (resumen)

Todos en inglés, formato `{domain}_{action}`:

- **Staff:** `staff_list`, `staff_get`, `staff_create`, `staff_update`, `staff_deactivate`, `staff_reactivate`, `staff_places_set`, `staff_rest_days_get`, `staff_rest_days_set`, `staff_absences_list`, `staff_absences_create`, `staff_absences_update`, `staff_absences_delete`.
- **Places:** `places_list`, `places_get`, `places_create`, `places_update`, `places_deactivate`, `places_reactivate`, `places_recommended_task`.
- **Cleaning:** `cleaning_list_by_date`, `cleaning_get`, `cleaning_create`, `cleaning_update`, `cleaning_set_status`, `cleaning_set_staff`, `cleaning_suggest_staff`.
- **Incidents:** `incidents_list`, `incidents_create`, `incidents_set_status`.
- **Reports:** `reports_services_by_staff`, `reports_average_times`, `reports_absences_by_month`, `reports_daily_sheet`.
- **Export:** `export_csv`, `export_xlsx`, `export_pdf` (reciben `kind`, `filters` y `dest_path`).
- **Backup:** `backup_create`, `backup_restore`.
- **Settings:** `settings_get`, `settings_update`.
- **Startup:** `take_startup_notice`.

**Contratos clave:**
- `cleaning_create` incluye obligatoriamente `staff: [{ staff_id, is_lead }]` (≥1).
- `cleaning_set_staff` recibe la lista completa; no admite vacío; máximo un `is_lead`.
- `staff_places_set` reemplaza el conjunto de apartamentos principales (solo apartamentos activos).
- `places_recommended_task(place_id, date)` devuelve `{ effective_status, recommended_service_type }`.
- `cleaning_suggest_staff(place_id, service_type, date, count)` devuelve candidatos ordenados con motivo (`count` por defecto 10).
- `export_*` reciben `{ kind, filters, dest_path }`; el frontend abre el diálogo `save` y pasa la ruta.
- `backup_create(dest_path)` / `backup_restore(src_path)`.

### 6.5 Exportación (D3)

**Flujo:**
1. El frontend arma la solicitud (`kind` + filtros) y abre un diálogo **Guardar como** (`@tauri-apps/plugin-dialog`).
2. Invoca el comando Rust correspondiente con `dest_path`.
3. Rust genera el archivo y lo escribe; devuelve la ruta o un `CommandError`.

**Formatos:**
- **CSV (datos):** `csv` crate; encabezados en español; UTF-8 con BOM; sin formato.
- **XLSX (planilla):** `rust_xlsxwriter`; título, fecha, encabezados, bordes, anchos, totales, configuración de página (horizontal, ajustar a ancho, márgenes), encabezado/pie.
- **PDF (planilla):** `printpdf`; misma data que XLSX; tabla con saltos de página, numeración y fuente embebida `DejaVuSans` para acentos/ñ.

**Tipos de exportación (`kind`):**
- `daily_sheet` (fecha)
- `services_by_staff` (rango)
- `average_times` (rango)
- `absences_by_month` (mes)

`export_csv` usa los mismos `kind`, pero genera datos tabulares sin formato.

### 6.6 Backup / Restore (D14)

**Backup `backup_create(dest_path)`:**
1. Crear directorio temporal en `app_cache_dir()`.
2. `VACUUM INTO temp/housekeeping.db` (copia consistente con WAL).
3. Copiar `photos/` a `temp/photos/`.
4. Escribir `manifest.json`.
5. Comprimir a `dest_path.tmp` y renombrar a `dest_path` (nunca sobrescribir a medias).
6. Limpiar el temporal.

**Restore `backup_restore(src_path)`:**
1. Validar que el ZIP contiene `housekeeping.db`, `photos/` y `manifest.json`.
2. Extraer a un temporal y validar la DB: cabecera SQLite, tablas/columnas esperadas, `integrity_check`, `foreign_key_check`.
3. Crear backup de seguridad de la DB actual (`VACUUM INTO`) y de las fotos.
4. `DbState` write-lock: cerrar el pool.
5. Reemplazar DB y fotos; reabrir con `connect()`; ejecutar migraciones; reemplazar el pool.
6. Si algo falla, restaurar la copia de seguridad y devolver un `CommandError`.
7. Devolver el resultado; el frontend muestra un mensaje y recarga los datos.

**Arranque:** `open_with_recovery` + `StartupNotice` + `quarantine_database` (patrón del proyecto hermano).

### 6.7 Configuración de Tauri

- **Plugins:** `tauri-plugin-single-instance`, `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-opener` (existente).
- **`capabilities/default.json`:** `core:default`, `opener:default`, `dialog:default`, `fs:default`.
- **`assetProtocol`:** habilitado con scope `["$APPDATA/photos/**"]` para mostrar fotos con `convertFileSrc`.
- **Ventana:** título en español (ej. “Gestión de Limpieza”); `lang="es"` en `app.html`.
- **Paquetes JS:** `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs` (además de `@tauri-apps/api` y `@tauri-apps/plugin-opener`).
- **Instancia única:** enfocar/restaurar la ventana existente al abrir una segunda instancia.

### 6.8 Calidad

- **Rust:** `cargo fmt` y `cargo clippy -- -D warnings`; tests con `cargo test`.
- **Frontend:** `bun run check` (svelte-check); `prettier` + `prettier-plugin-svelte`; ESLint con `eslint-plugin-svelte` (o Biome).
- **Scripts:** `check`, `lint`, `format`, `test` en `package.json`.
- **Git:** commit automático por feature/cambio grande con mensaje en inglés; `CHANGELOG.md` post-MVP (ver sección 11).
- **Tipos compartidos:** DTOs TS manuales en `types.ts`; opcional futuro `ts-rs`/`specta` para generarlos desde Rust.

---

## 7. Testing

### Backend (obligatorio)

- **Unit tests:** helpers de fechas, temporadas, disponibilidad, ranking, `effective_status`, recomendación (incluye solapes y fechas parciales), normalización/validaciones.
- **Integración tests:** SQLite `:memory:` o temporal; migraciones aplicadas; CRUD por dominio; unicidad de `national_id` y `place.name`; soft delete/reactivación; `staff_place` solo apartamentos; `is_lead` único; ausencias solapadas; servicio sin personal prohibido; trigger del último `service_staff`; settings; exportaciones; backup/restore round-trip.
- **Casos clave:**
  - Zafral dentro/fuera de temporada.
  - Descanso y ausencia (inclusive en los extremos).
  - Recomendación: checkout hoy, check-in en 2 días, ocupado, sin fechas, fecha parcial, `cancelled`.
  - Servicio: transiciones válidas/ inválidas, completar sin horas, quitar último funcionario.
  - Baja de funcionario con servicio futuro único: bloquea.
  - Backup corrupto: rechaza; restore con fallo: recupera la DB anterior.
  - Exportaciones: archivos no vacíos y cabeceras correctas.

### Frontend

- `bun run check` como gate de tipos.
- *(Opcional, fase posterior):* Vitest para lógica de servicios.
- Verificación manual de UI/UX: flujo completo de creación de servicio, navegación por teclado, foco visible, estados vacío/cargando/error, tema claro/oscuro, ausencia de parpadeo y confirmaciones de acciones destructivas.

---

## 8. Roadmap por fases

- [x] **Fase 0 — Fundación.** Dependencias Rust y JS, `db.rs` (connect, PRAGMAs, `open_with_recovery`), `state.rs`, `error.rs`, migración inicial, `settings`, plugins/capabilities/single-instance, `assetProtocol`, tokens CSS + arranque de tema, `app.html` en español, `labels.ts`/`errors.ts`. **Commit inicial.** Tests de arranque + migración + settings.
- [x] **Fase 1 — Personal.** CRUD + baja/reactivación, roles, `gender`, zafral + temporada, descansos, ausencias, apartamentos principales, foto + avatares. Tests.
- [x] **Fase 2 — Lugares.** CRUD + baja/reactivación, estados (incl. `cancelled`), fechas opcionales, tarea recomendada. Tests.
- [x] **Fase 3 — Servicios.** CRUD con personal obligatorio, transiciones, disponibilidad, asignación asistida. Tests.
- [x] **Fase 4 — Tablero diario.** Pantalla principal, selector de fecha, pendientes sugeridos, personal, incidencias, toggle de tema y **UX pulida** (estados, atajos, feedback, accesibilidad).
- [x] **Fase 5 — Incidencias + Reportes.** CRUD incidencias; reportes (servicios por persona, tiempos promedio, ausencias por mes).
- [ ] **Fase 6 — Exportación + Backup/restore + Ajustes.** CSV/XLSX/PDF en Rust; backup ZIP; restore con swap; pantalla de configuración global con `settingsService`.
- [ ] **Fase 7 — Pulido y release del MVP.** Validaciones finales, mensajes de error en español, **revisión UX/UI con `frontend-design` y `accessibility`**, rendimiento, accesibilidad básica, documentación, **`CHANGELOG.md` con la entrada `0.1.0`**, commit final, `cargo test` + `cargo clippy` + `bun run check` + `bun run build` en verde.

**Fase posterior (fuera del MVP):** inventario de insumos.

---

## 9. Decisiones finales y cambios respecto al plan original

### Agregadas
1. **`gender`** como campo de personal (avatar por defecto hombre/mujer).
2. **Avatares de stock** `default-male.svg` / `default-female.svg` + `Avatar.svelte`.
3. **`is_seasonal` + `season_start_date` / `season_end_date`** (zafral con temporada).
4. **`staff_place`** (N:M) para varios apartamentos principales; no restringe asignaciones a otros lugares.
5. **`check_in_date` / `check_out_date`** con `NULL = desconocido` y reglas de recomendación.
6. **`cancelled`** como estado manual de apartamento (turno omitido).
7. **Personal obligatorio en servicios** (D15).
8. **Tabla `settings` + `settingsService.svelte.ts`** (D13).
9. **Exportación en Rust** (CSV/XLSX/PDF) (D3).
10. **Backup ZIP** con DB + fotos + manifest, validación y swap del pool (D14).
11. **Baja lógica + reactivación** de personal y lugares.
12. **Búsqueda/filtros**, `notes` y timestamps.
13. **Tema claro/oscuro** con persistencia.
14. **Interfaz 100 % en español** y `labels.ts`.
15. **Instancia única**, plugins y permisos de Tauri.
16. **`CHANGELOG.md` en inglés** (Keep a Changelog + SemVer) para registrar cambios posteriores al MVP.
17. **Commit automático** tras cada feature/cambio grande, con mensaje breve en inglés (Conventional Commits).
18. **Interfaz moderna, rápida e intuitiva** (D19), con criterios de UX/UI, rendimiento y accesibilidad.

### Mejoradas
1. **Estados de apartamento** con `cancelled`, `NULL = automático` y override manual.
2. **Fechas de apartamento** con recomendación priorizada y soporte de fechas parciales.
3. **Disponibilidad** con temporada, descansos, ausencias y solapes de servicio.
4. **Exportación** unificada: CSV datos + XLSX/PDF planillas, todo en Rust.
5. **Backup/restore** alineado con el proyecto hermano (VACUUM INTO, validación, safety, swap de pool).
6. **Servicios frontend como clases** con runes.

### Excluidas / diferidas
1. **Inventario de insumos** → fase posterior.
2. **Impresión del navegador / print CSS** → reemplazada por XLSX/PDF en Rust.
3. **Categoría de asignación alternativa** → eliminada; solo apartamentos principales + asignación libre a otros lugares.

### Decisiones menores (valores por defecto)
- Etiquetas exactas de estados: las propuestas en 3.2.
- Ventanas de recomendación: configurables, por defecto 2 días.
- `gender`: obligatorio `male`/`female`.
- Foto de incidencia: opcional.
- Temporada zafral: si se cargan fechas, ambas y `start <= end`; si no, informativa.

---

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|--------|------------|
| Backup con WAL activo | `VACUUM INTO` + validación + ZIP; nunca copiar el `.db` a mano. |
| Restore deja la app sin DB | Safety backup + reapertura + restauración automática (patrón del hermano). |
| Dos instancias escribiendo | `tauri-plugin-single-instance`. |
| Fotos no visibles en el WebView | `assetProtocol` con scope + `convertFileSrc` + rutas resueltas por el backend. |
| PDF con acentos/ñ | Fuente `DejaVuSans` embebida; probar UTF-8 en XLSX/CSV. |
| Servicios sin personal por bajas | Bloqueo de baja si es el único asignado; trigger de último `service_staff`. |
| Recomendaciones inconsistentes | Función única `effective_status` + prioridad documentada + tests. |
| Crecimiento del binario por PDF | `printpdf` en lugar de `typst`; medir tamaño en Fase 6. |

---

## 11. Proceso de cambios, commits y changelog

### 11.1 Commits

- El repositorio no tiene commits iniciales: crear el **commit inicial** al comenzar la Fase 0 (código base + plan).
- Después de cada **feature completa o cambio grande**, hacer `git add` de los archivos relacionados y un **commit automático**.
- Mensajes **breves, descriptivos y en inglés**, en modo imperativo y con **Conventional Commits**:
  - `feat: add staff CRUD and photo upload`
  - `feat: implement assisted staff assignment`
  - `feat: add daily board and theme toggle`
  - `feat: add CSV, XLSX and PDF exports`
  - `fix: prevent services without assigned staff`
  - `chore: add database migrations and settings`
  - `docs: update changelog for 0.1.1`
- Un commit por unidad lógica; no mezclar cambios no relacionados.
- Antes de commitear, ejecutar los checks aplicables (`cargo test`, `cargo clippy`, `bun run check`, `bun run build`).
- No commitear artefactos (`build/`, `.svelte-kit/`, `target/`, `node_modules/`); ya están ignorados.

### 11.2 Changelog post-MVP

- `CHANGELOG.md` está en inglés y sigue **Keep a Changelog 1.1.0** + **Semantic Versioning**.
- **A partir del MVP**, todo cambio visible para el usuario debe registrarse bajo `[Unreleased]` con la categoría correspondiente: `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.
- Al cerrar una versión, mover las entradas a `## [x.y.z] - YYYY-MM-DD`, crear un nuevo `[Unreleased]` y actualizar la versión en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`.
- Los cambios internos (refactors, tests, tooling) pueden omitir el changelog, pero igualmente se commitean.
- El MVP se registrará como **`0.1.0`** al completarse la Fase 7.

### 11.3 Definition of Done por feature

1. Implementación completa y modular.
2. Tests/checks aplicables en verde.
3. `CHANGELOG.md` actualizado (solo post-MVP).
4. Commit en inglés con mensaje descriptivo.
