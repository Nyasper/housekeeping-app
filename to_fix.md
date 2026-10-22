# Pre-MVP fixes

Checklist de los hallazgos de la revisión. Se trabaja de arriba hacia abajo, un
commit por punto después de correr los checks correspondientes.

## Alta

- [x] **1. Fecha de referencia obsoleta en Lugares y Servicios.**
  `placesService.referenceDate` persiste la última fecha del tablero y `load()`
  sin fecha la reutiliza. Lugares muestra “Tarea recomendada hoy” con datos de
  otra fecha; Servicios no recarga lugares al cambiar de día y usa una
  recomendación vieja como tipo por defecto.
  Archivos: `placesService.svelte.ts`, `routes/places/+page.svelte`,
  `routes/cleaning/+page.svelte`, `ServiceForm.svelte`.

- [x] **2. Personal dado de baja invisible en el formulario de servicio.**
  `AvailabilityList` solo renderiza sugerencias y personal activo; un asignado
  inactivo queda seleccionado pero sin tarjeta, y no se puede quitar.
  Archivos: `AvailabilityList.svelte`, `ServiceForm.svelte`, `routes/+page.svelte`,
  `routes/cleaning/+page.svelte`.

- [x] **3. No se puede reabrir un servicio cancelado desde la UI.**
  El backend exige `confirm_reopen` para `cancelled → planned`, pero el checkbox
  solo se muestra para servicios completados.
  Archivos: `ServiceForm.svelte`.

## Media

- [x] **4. Aviso falso de doble asignación.**
  Se advierte “ya tiene N servicio(s) ese día” aunque las horas estén cargadas y
  no se solapen. `PLAN.md:243` pide avisar solo por solapamiento, o por mismo día
  cuando faltan horas.
  Archivo: `src-tauri/src/domain/cleaning/service.rs`.

- [x] **5. `completed → cancelled` permitido con confirmación.**
  `completed` es terminal en `PLAN.md:212-213`; solo debería reabrirse a
  `planned`/`in_progress`.
  Archivo: `src-tauri/src/domain/cleaning/service.rs` (+ opciones del
  `ServiceForm`).

- [x] **6. CSV poco compatible con Excel en es-UY.**
  Usa coma como delimitador y punto decimal; Excel en español espera `;` y coma
  decimal. `PLAN.md:273` pide UTF-8 con BOM para Excel.
  Archivos: `export_csv.rs`, `queries.rs` (`to_csv_string`).

- [x] **7. Editar un funcionario borra sus lugares principales desactivados.**
  `staff_places_set` reemplaza todas las asociaciones con la lista de lugares
  activos que envía el formulario.
  Archivos: `staff/service.rs`, `staff/repo.rs`, `PLAN.md`.

## Baja / casos borde

- [x] **8. Totales de la planilla diaria exportada incluyen cancelados.**
  La UI excluye cancelados de los KPIs; el export los cuenta y suma sus minutos.
  Archivo: `src-tauri/src/domain/reports/queries.rs`.

- [x] **9. “Cancelado desde” usa `updated_at`.**
  Es la última modificación, no la cancelación, y está en UTC. Agregar
  `cancelled_at` y formatear timestamps UTC a local.
  Archivos: migración, `places` model/repo/service, `DailyBoard.svelte`,
  `routes/places/+page.svelte`, `format.ts`, `types.ts`, `factories.ts`,
  `PLAN.md`.

- [x] **10. `update_service` permite cambiar a un lugar desactivado.**
  `create_service` lo rechaza; `update_service` no. Mantener el lugar histórico
  inactivo pero rechazar el cambio a otro lugar inactivo.
  Archivo: `src-tauri/src/domain/cleaning/service.rs`.

- [ ] **11. `cleaning_set_staff` modifica servicios completados sin confirmación.**
  Agregar `confirm_reopen` y validar la transición, o bloquear si está completado.
  Archivos: `cleaning/service.rs`, `cleaning/commands.rs`,
  `cleaningService.svelte.ts`.

- [ ] **12. PDF: una fila muy alta desborda el pie de página.**
  Limitar la altura de una fila (truncar con elipsis) para que siempre entre en
  una página.
  Archivo: `src-tauri/src/domain/reports/export_pdf.rs`.

- [ ] **13. `cleaningService.setStatus` no puede limpiar horas.**
  `startTime ?? service.start_time` impide pasar `null` para borrar.
  Archivo: `cleaningService.svelte.ts`.

- [ ] **14. Restore reporta error si falla una recarga posterior.**
  La restauración puede haber sido exitosa aunque `settingsService.load` u otra
  recarga fallen.
  Archivo: `backupService.svelte.ts`.
