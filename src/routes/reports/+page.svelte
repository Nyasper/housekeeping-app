<script lang="ts">
  import { onMount } from 'svelte';

  import {
    absenceTypeLabels,
    labelOf,
    serviceStatusLabels,
    serviceTypeLabels
  } from '#lib/api/labels.js';
  import type { ExportFilters, ExportFormat, ExportKind } from '#lib/api/types.js';
  import EmptyState from '#lib/components/EmptyState.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { dateService } from '#lib/services/dateService.svelte.js';
  import { exportService } from '#lib/services/exportService.svelte.js';
  import { reportsService } from '#lib/services/reportsService.svelte.js';
  import {
    currentMonthIso,
    currentMonthRangeIso,
    formatDate,
    formatDurationMinutes,
    formatMonth,
    formatNumber,
    formatTime
  } from '#lib/utils/format.js';

  const initialRange = currentMonthRangeIso();

  let activeTab = $state<ExportKind>('services_by_staff');
  let rangeFrom = $state(initialRange.from);
  let rangeTo = $state(initialRange.to);
  let selectedMonth = $state(currentMonthIso());
  let sheetDate = $state(dateService.selectedDate);
  let toastMessage = $state<string | null>(null);
  let toastTone = $state<'success' | 'warning' | 'error'>('success');

  const rangeInvalid = $derived(
    Boolean(rangeFrom && rangeTo && rangeFrom > rangeTo)
  );

  const currentExportFilters = $derived<ExportFilters>({
    date: sheetDate,
    from: rangeFrom,
    to: rangeTo,
    month: selectedMonth
  });

  const canExportCurrentTab = $derived.by(() => {
    if (exportService.exporting || reportsService.loading) {
      return false;
    }
    if (activeTab === 'services_by_staff' || activeTab === 'average_times') {
      return Boolean(rangeFrom && rangeTo && !rangeInvalid);
    }
    if (activeTab === 'absences_by_month') {
      return Boolean(selectedMonth);
    }
    return Boolean(sheetDate);
  });

  async function handleExport(format: ExportFormat): Promise<void> {
    if (!canExportCurrentTab) {
      return;
    }
    toastMessage = null;
    exportService.clearError();

    const savedPath = await exportService.exportReport(
      activeTab,
      format,
      currentExportFilters
    );

    if (savedPath) {
      const formatLabel = format.toUpperCase();
      toastTone = 'success';
      toastMessage = `Archivo ${formatLabel} exportado en «${savedPath}».`;
    } else if (exportService.error) {
      toastTone = 'error';
      toastMessage = exportService.error;
    }
  }

  async function loadActiveTab(): Promise<void> {
    if (activeTab === 'services_by_staff' || activeTab === 'average_times') {
      if (rangeInvalid || !rangeFrom || !rangeTo) {
        return;
      }
      await reportsService.loadRangeReports({ from: rangeFrom, to: rangeTo });
    } else if (activeTab === 'absences_by_month') {
      if (!selectedMonth) {
        return;
      }
      await reportsService.loadAbsencesByMonth(selectedMonth);
    } else if (activeTab === 'daily_sheet') {
      if (!sheetDate) {
        return;
      }
      await reportsService.loadDailySheet(sheetDate);
    }
  }

  onMount(() => {
    void Promise.all([
      reportsService.loadRangeReports({ from: rangeFrom, to: rangeTo }),
      reportsService.loadAbsencesByMonth(selectedMonth),
      reportsService.loadDailySheet(sheetDate)
    ]);
  });

  async function selectTab(tab: ExportKind): Promise<void> {
    activeTab = tab;
    await loadActiveTab();
  }

  async function resetCurrentMonthRange(): Promise<void> {
    const current = currentMonthRangeIso();
    rangeFrom = current.from;
    rangeTo = current.to;
    await reportsService.loadRangeReports({ from: rangeFrom, to: rangeTo });
  }

  async function resetCurrentMonth(): Promise<void> {
    selectedMonth = currentMonthIso();
    await reportsService.loadAbsencesByMonth(selectedMonth);
  }

  async function resetSheetToday(): Promise<void> {
    dateService.goToToday();
    sheetDate = dateService.selectedDate;
    await reportsService.loadDailySheet(sheetDate);
  }

  const dailySheetActiveCount = $derived(
    reportsService.dailySheet.filter((row) => row.status !== 'cancelled').length
  );

  const dailySheetCompletedCount = $derived(
    reportsService.dailySheet.filter((row) => row.status === 'completed').length
  );

  const dailySheetOpenIncidents = $derived(
    reportsService.dailySheet.reduce((acc, row) => acc + row.open_incidents_count, 0)
  );
</script>

<svelte:head>
  <title>Reportes · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">Historial, estadísticas y documentos imprimibles</p>
    <h1 class="page-title">Reportes</h1>
    <p class="page-lede">
      Consultá servicios y horas por funcionario, tiempos promedio por tipo de tarea, ausencias
      mensuales y la planilla operativa diaria. Exportá cualquier vista en CSV, XLSX o PDF.
    </p>
  </div>
</header>

<Toast
  message={toastMessage}
  tone={toastTone}
  ondismiss={() => {
    toastMessage = null;
  }}
/>

<div class="tabs-bar panel" role="tablist" aria-label="Tipos de reporte">
  <button
    type="button"
    role="tab"
    id="tab-services-by-staff"
    aria-controls="panel-services-by-staff"
    aria-selected={activeTab === 'services_by_staff'}
    class={`tab-button ${activeTab === 'services_by_staff' ? 'is-active' : ''}`}
    onclick={() => selectTab('services_by_staff')}
  >
    Servicios por persona
  </button>
  <button
    type="button"
    role="tab"
    id="tab-average-times"
    aria-controls="panel-average-times"
    aria-selected={activeTab === 'average_times'}
    class={`tab-button ${activeTab === 'average_times' ? 'is-active' : ''}`}
    onclick={() => selectTab('average_times')}
  >
    Tiempos promedio
  </button>
  <button
    type="button"
    role="tab"
    id="tab-absences-by-month"
    aria-controls="panel-absences-by-month"
    aria-selected={activeTab === 'absences_by_month'}
    class={`tab-button ${activeTab === 'absences_by_month' ? 'is-active' : ''}`}
    onclick={() => selectTab('absences_by_month')}
  >
    Ausencias por mes
  </button>
  <button
    type="button"
    role="tab"
    id="tab-daily-sheet"
    aria-controls="panel-daily-sheet"
    aria-selected={activeTab === 'daily_sheet'}
    class={`tab-button ${activeTab === 'daily_sheet' ? 'is-active' : ''}`}
    onclick={() => selectTab('daily_sheet')}
  >
    Planilla diaria
  </button>
</div>

<section class="filters-bar panel" aria-label="Período y exportación del reporte">
  <div class="filters-row">
    <div class="filter-group">
      {#if activeTab === 'services_by_staff' || activeTab === 'average_times'}
        <div class="filter-field">
          <label for="report-range-from">Desde</label>
          <input
            id="report-range-from"
            class="input"
            type="date"
            bind:value={rangeFrom}
            onchange={loadActiveTab}
            aria-invalid={rangeInvalid}
          />
        </div>
        <div class="filter-field">
          <label for="report-range-to">Hasta</label>
          <input
            id="report-range-to"
            class="input"
            type="date"
            bind:value={rangeTo}
            onchange={loadActiveTab}
            aria-invalid={rangeInvalid}
          />
        </div>
        <button
          type="button"
          class="button button-secondary"
          onclick={resetCurrentMonthRange}
          disabled={reportsService.loading}
        >
          Mes actual
        </button>
      {:else if activeTab === 'absences_by_month'}
        <div class="filter-field">
          <label for="report-month">Mes a consultar</label>
          <input
            id="report-month"
            class="input"
            type="month"
            bind:value={selectedMonth}
            onchange={loadActiveTab}
          />
        </div>
        <button
          type="button"
          class="button button-secondary"
          onclick={resetCurrentMonth}
          disabled={reportsService.loading}
        >
          Mes actual
        </button>
      {:else}
        <div class="filter-field">
          <label for="report-sheet-date">Fecha de la planilla</label>
          <input
            id="report-sheet-date"
            class="input"
            type="date"
            bind:value={sheetDate}
            onchange={loadActiveTab}
          />
        </div>
        <button
          type="button"
          class="button button-secondary"
          onclick={resetSheetToday}
          disabled={reportsService.loading}
        >
          Hoy
        </button>
      {/if}
    </div>

    <div class="export-actions" role="group" aria-label="Exportar reporte activo">
      <button
        type="button"
        class="button button-secondary"
        onclick={() => handleExport('csv')}
        disabled={!canExportCurrentTab}
      >
        {exportService.exporting && exportService.activeFormat === 'csv'
          ? 'Exportando CSV…'
          : 'Exportar CSV'}
      </button>
      <button
        type="button"
        class="button button-secondary"
        onclick={() => handleExport('xlsx')}
        disabled={!canExportCurrentTab}
      >
        {exportService.exporting && exportService.activeFormat === 'xlsx'
          ? 'Generando XLSX…'
          : 'Planilla XLSX'}
      </button>
      <button
        type="button"
        class="button button-primary"
        onclick={() => handleExport('pdf')}
        disabled={!canExportCurrentTab}
      >
        {exportService.exporting && exportService.activeFormat === 'pdf'
          ? 'Generando PDF…'
          : 'Planilla PDF'}
      </button>
    </div>
  </div>

  {#if (activeTab === 'services_by_staff' || activeTab === 'average_times') && rangeInvalid}
    <p class="range-error" role="alert">
      La fecha de inicio no puede ser posterior a la fecha de fin.
    </p>
  {/if}
</section>

{#if reportsService.error}
  <section class="panel error-banner" role="alert">
    <p>{reportsService.error}</p>
    <button type="button" class="button button-secondary" onclick={loadActiveTab}>
      Reintentar
    </button>
  </section>
{/if}

{#if activeTab === 'services_by_staff'}
  <div
    id="panel-services-by-staff"
    role="tabpanel"
    aria-labelledby="tab-services-by-staff"
    class="report-panel panel"
  >
    <div class="report-head">
      <div>
        <h2>Servicios por persona</h2>
        <p>
          Período: {formatDate(rangeFrom)} al {formatDate(rangeTo)} · Excluye servicios cancelados.
        </p>
      </div>
      <div class="kpi-pills">
        <span class="kpi-pill">
          Total asignaciones: <strong>{formatNumber(reportsService.totalServicesInRange)}</strong>
        </span>
        <span class="kpi-pill">
          Horas registradas: <strong>{formatNumber(reportsService.totalHoursInRange)} h</strong>
        </span>
      </div>
    </div>

    {#if reportsService.loading && reportsService.servicesByStaff.length === 0}
      <p class="loading-state" role="status">Calculando servicios por funcionario…</p>
    {:else if reportsService.servicesByStaff.length === 0}
      <EmptyState
        title="Sin datos de personal en este rango"
        description="No hay personal activo ni servicios registrados entre las fechas seleccionadas."
      />
    {:else}
      <div class="table-wrap">
        <table class="report-table">
          <thead>
            <tr>
              <th scope="col">Funcionario</th>
              <th scope="col" class="num-col">Servicios asignados</th>
              <th scope="col" class="num-col">Horas registradas</th>
            </tr>
          </thead>
          <tbody>
            {#each reportsService.servicesByStaff as row (row.staff_id)}
              <tr>
                <td class="primary-cell">{row.staff_name}</td>
                <td class="num-col">{formatNumber(row.service_count)}</td>
                <td class="num-col">{formatNumber(row.total_hours)} h</td>
              </tr>
            {/each}
          </tbody>
          <tfoot>
            <tr>
              <th scope="row">Total del período</th>
              <td class="num-col">
                <strong>{formatNumber(reportsService.totalServicesInRange)}</strong>
              </td>
              <td class="num-col">
                <strong>{formatNumber(reportsService.totalHoursInRange)} h</strong>
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    {/if}
  </div>
{:else if activeTab === 'average_times'}
  <div
    id="panel-average-times"
    role="tabpanel"
    aria-labelledby="tab-average-times"
    class="report-panel panel"
  >
    <div class="report-head">
      <div>
        <h2>Tiempos promedio por tipo de servicio</h2>
        <p>
          Período: {formatDate(rangeFrom)} al {formatDate(rangeTo)} · Solo servicios completados con
          hora de inicio y fin.
        </p>
      </div>
      <div class="kpi-pills">
        <span class="kpi-pill">
          Servicios completados: <strong>{formatNumber(reportsService.totalCompletedServices)}</strong>
        </span>
      </div>
    </div>

    {#if reportsService.loading && reportsService.averageTimes.length === 0}
      <p class="loading-state" role="status">Calculando tiempos promedio…</p>
    {:else if reportsService.averageTimes.length === 0}
      <EmptyState
        title="Sin tiempos promedio disponibles"
        description="Completá servicios con hora de inicio y fin para ver las duraciones promedio."
      />
    {:else}
      <div class="table-wrap">
        <table class="report-table">
          <thead>
            <tr>
              <th scope="col">Tipo de servicio</th>
              <th scope="col" class="num-col">Servicios completados</th>
              <th scope="col" class="num-col">Promedio (minutos)</th>
              <th scope="col" class="num-col">Duración promedio</th>
            </tr>
          </thead>
          <tbody>
            {#each reportsService.averageTimes as row (row.service_type)}
              <tr>
                <td class="primary-cell">{labelOf(serviceTypeLabels, row.service_type)}</td>
                <td class="num-col">{formatNumber(row.completed_services)}</td>
                <td class="num-col">
                  {row.completed_services > 0 ? `${formatNumber(row.average_minutes)} min` : '—'}
                </td>
                <td class="num-col">
                  {row.completed_services > 0
                    ? formatDurationMinutes(row.average_minutes)
                    : 'Sin servicios completados'}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
{:else if activeTab === 'absences_by_month'}
  <div
    id="panel-absences-by-month"
    role="tabpanel"
    aria-labelledby="tab-absences-by-month"
    class="report-panel panel"
  >
    <div class="report-head">
      <div>
        <h2>Ausencias por mes</h2>
        <p>
          Mes: {formatMonth(selectedMonth)} · Los rangos que cruzan meses se reparten por día dentro
          del mes consultado.
        </p>
      </div>
      <div class="kpi-pills">
        <span class="kpi-pill">
          Total días de ausencia: <strong>{formatNumber(reportsService.totalAbsenceDays)}</strong>
        </span>
      </div>
    </div>

    {#if reportsService.loading && reportsService.absencesByMonth.length === 0}
      <p class="loading-state" role="status">Calculando ausencias del mes…</p>
    {:else if reportsService.absencesByMonth.length === 0}
      <EmptyState
        title="Sin ausencias registradas en este mes"
        description={`No hay certificaciones médicas, licencias ni faltas que cubran días de ${formatMonth(selectedMonth)}.`}
      />
    {:else}
      <div class="table-wrap">
        <table class="report-table">
          <thead>
            <tr>
              <th scope="col">Funcionario</th>
              <th scope="col">Tipo de ausencia</th>
              <th scope="col" class="num-col">Días en el mes</th>
            </tr>
          </thead>
          <tbody>
            {#each reportsService.absencesByMonth as row (`${row.staff_id}-${row.type}`)}
              <tr>
                <td class="primary-cell">{row.staff_name}</td>
                <td>{labelOf(absenceTypeLabels, row.type)}</td>
                <td class="num-col">{formatNumber(row.days)}</td>
              </tr>
            {/each}
          </tbody>
          <tfoot>
            <tr>
              <th scope="row" colspan="2">Total de días en {formatMonth(selectedMonth)}</th>
              <td class="num-col">
                <strong>{formatNumber(reportsService.totalAbsenceDays)}</strong>
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    {/if}
  </div>
{:else}
  <div
    id="panel-daily-sheet"
    role="tabpanel"
    aria-labelledby="tab-daily-sheet"
    class="report-panel panel"
  >
    <div class="report-head">
      <div>
        <h2>Planilla diaria</h2>
        <p>
          Fecha: {formatDate(sheetDate)} · Detalle de lugares, personal asignado, horarios, estados
          e incidencias.
        </p>
      </div>
      <div class="kpi-pills">
        <span class="kpi-pill">
          Activos/Completados: <strong>{dailySheetCompletedCount}/{dailySheetActiveCount}</strong>
        </span>
        <span class="kpi-pill">
          Incidencias abiertas: <strong>{dailySheetOpenIncidents}</strong>
        </span>
      </div>
    </div>

    {#if reportsService.loading && reportsService.dailySheet.length === 0}
      <p class="loading-state" role="status">Cargando planilla diaria…</p>
    {:else if reportsService.dailySheet.length === 0}
      <EmptyState
        title="Sin servicios registrados en esta fecha"
        description="No hay servicios programados ni completados para la fecha seleccionada."
      />
    {:else}
      <div class="table-wrap">
        <table class="report-table">
          <thead>
            <tr>
              <th scope="col">Lugar</th>
              <th scope="col">Tipo</th>
              <th scope="col">Estado</th>
              <th scope="col">Personal asignado</th>
              <th scope="col">Horario</th>
              <th scope="col">Incidencias</th>
            </tr>
          </thead>
          <tbody>
            {#each reportsService.dailySheet as row (row.service_id)}
              <tr>
                <td class="primary-cell">
                  <div>{row.place_name}</div>
                  {#if row.place_location}
                    <div class="cell-sub">{row.place_location}</div>
                  {/if}
                </td>
                <td>{labelOf(serviceTypeLabels, row.service_type)}</td>
                <td>
                  <span class={`badge badge-status-${row.status}`}>
                    {labelOf(serviceStatusLabels, row.status)}
                  </span>
                </td>
                <td>{row.staff_names.join(', ')}</td>
                <td>
                  {#if row.start_time && row.end_time}
                    {formatTime(row.start_time)} – {formatTime(row.end_time)}
                    {#if row.duration_minutes !== null}
                      <div class="cell-sub">{formatDurationMinutes(row.duration_minutes)}</div>
                    {/if}
                  {:else if row.start_time}
                    Inicio {formatTime(row.start_time)}
                  {:else}
                    Sin horario
                  {/if}
                </td>
                <td>
                  {#if row.incidents_summary.length === 0}
                    <span class="text-muted">Sin incidencias</span>
                  {:else}
                    <ul class="incidents-mini-list">
                      {#each row.incidents_summary as item (item)}
                        <li>{item}</li>
                      {/each}
                    </ul>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
{/if}

<style>
  .tabs-bar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .tab-button {
    flex: 1 1 11rem;
    min-height: 2.75rem;
    padding: 0.65rem 1rem;
    border: 1px solid transparent;
    border-radius: var(--radius-lg);
    background: transparent;
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    font-weight: 650;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast),
      border-color var(--transition-fast);
  }

  .tab-button:hover {
    background: var(--color-surface-muted);
    color: var(--color-ink);
  }

  .tab-button.is-active {
    background: var(--color-primary);
    color: var(--color-on-primary);
    box-shadow: var(--shadow-sm);
  }

  .filters-bar {
    padding: var(--space-4) var(--space-5);
    margin-bottom: var(--space-5);
  }

  .filters-row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .filter-group {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-3);
  }

  .filter-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .filter-field label {
    font-size: var(--text-xs);
    font-weight: 650;
    color: var(--color-ink-soft);
  }

  .filter-field .input {
    width: auto;
    min-width: 11rem;
  }

  .export-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .range-error {
    margin-top: var(--space-3);
    color: var(--color-danger);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .error-banner {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-5);
    margin-bottom: var(--space-5);
    border-color: var(--color-danger);
    background: var(--color-danger-soft);
  }

  .report-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-6);
  }

  .report-head {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-line);
  }

  .report-head h2 {
    font-size: var(--text-xl);
  }

  .report-head p {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .kpi-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .kpi-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.45rem 0.85rem;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-pill);
    background: var(--color-surface-raised);
    font-size: var(--text-xs);
    color: var(--color-ink-soft);
  }

  .kpi-pill strong {
    color: var(--color-ink);
    font-size: var(--text-sm);
  }

  .loading-state {
    padding: var(--space-5) 0;
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .table-wrap {
    overflow-x: auto;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-lg);
  }

  .report-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
    text-align: left;
  }

  .report-table th,
  .report-table td {
    padding: 0.85rem 1rem;
    border-bottom: 1px solid var(--color-line);
    vertical-align: top;
  }

  .report-table thead th {
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
    font-weight: 700;
    letter-spacing: 0.01em;
  }

  .report-table tbody tr:last-child td {
    border-bottom: 0;
  }

  .report-table tbody tr:hover {
    background: var(--color-surface-raised);
  }

  .report-table tfoot th,
  .report-table tfoot td {
    border-top: 2px solid var(--color-line-strong);
    border-bottom: 0;
    background: var(--color-surface-raised);
    font-weight: 650;
  }

  .primary-cell {
    font-weight: 650;
  }

  .cell-sub {
    margin-top: 0.15rem;
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    font-weight: 400;
  }

  .num-col {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.65rem;
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .badge-status-planned {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-status-in_progress {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .badge-status-completed {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .badge-status-cancelled {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .incidents-mini-list {
    margin: 0;
    padding-left: 1rem;
    font-size: var(--text-xs);
  }

  @media (max-width: 720px) {
    .report-panel {
      padding: var(--space-4);
    }
  }
</style>
