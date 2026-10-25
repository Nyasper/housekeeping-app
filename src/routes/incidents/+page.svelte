<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  import { isTauriRuntime } from '#lib/api/invoke.js';
  import {
    incidentStatusLabels,
    incidentTypeLabels,
    labelOf,
    serviceTypeLabels,
    toOptions
  } from '#lib/api/labels.js';
  import type {
    Incident,
    IncidentInput,
    IncidentStatus,
    IncidentType
  } from '#lib/api/types.js';
  import EmptyState from '#lib/components/EmptyState.svelte';
  import IncidentForm from '#lib/components/IncidentForm.svelte';
  import Modal from '#lib/components/Modal.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { cleaningService } from '#lib/services/cleaningService.svelte.js';
  import { dateService } from '#lib/services/dateService.svelte.js';
  import { incidentsService } from '#lib/services/incidentsService.svelte.js';
  import { formatDate, formatUtcDateTimeDate } from '#lib/utils/format.js';

  const incidentTypeOptions = toOptions(incidentTypeLabels);
  const incidentStatusOptions = toOptions(incidentStatusLabels);

  let statusFilter = $state<IncidentStatus | 'all'>('all');
  let typeFilter = $state<IncidentType | 'all'>('all');
  let dateFilter = $state('');
  let serviceFilter = $state<number | 'all'>('all');
  let searchInput = $state('');
  let debouncedSearch = $state('');

  let showForm = $state(false);
  let incidentFormDirty = $state(false);
  let formServiceDate = $state(dateService.selectedDate);
  let previewIncident = $state<Incident | null>(null);

  let toastMessage = $state<string | null>(null);
  let toastTone = $state<'success' | 'warning' | 'error'>('success');
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  function handleSearchInput(event: Event): void {
    const value = (event.currentTarget as HTMLInputElement).value;
    searchInput = value;
    if (searchTimer) {
      clearTimeout(searchTimer);
    }
    searchTimer = setTimeout(() => {
      debouncedSearch = value.trim().toLocaleLowerCase('es-UY');
    }, 150);
  }

  async function loadIncidents(): Promise<void> {
    await incidentsService.load({
      status: statusFilter === 'all' ? null : statusFilter,
      type: typeFilter === 'all' ? null : typeFilter,
      date: dateFilter.trim() ? dateFilter.trim() : null,
      service_id: serviceFilter === 'all' ? null : serviceFilter
    });
  }

  onMount(() => {
    void Promise.all([
      loadIncidents(),
      cleaningService.loadByDate(dateService.selectedDate)
    ]);
  });

  // Distinct services referenced across loaded incidents or current day's services for the service filter
  const availableServiceFilterOptions = $derived.by(() => {
    const seen: Record<number, boolean> = {};
    const options: Array<{ id: number; label: string }> = [];
    for (const svc of cleaningService.items) {
      if (!seen[svc.id]) {
        seen[svc.id] = true;
        options.push({
          id: svc.id,
          label: `${svc.place_name} · ${labelOf(serviceTypeLabels, svc.service_type)} (${formatDate(svc.date)})`
        });
      }
    }
    for (const inc of incidentsService.items) {
      if (!seen[inc.service_id]) {
        seen[inc.service_id] = true;
        options.push({
          id: inc.service_id,
          label: `${inc.place_name} · ${labelOf(serviceTypeLabels, inc.service_type)} (${formatDate(inc.date)})`
        });
      }
    }
    return options;
  });

  const filteredIncidents = $derived.by(() => {
    if (!debouncedSearch) {
      return incidentsService.items;
    }
    return incidentsService.items.filter((incident) => {
      const placeMatch = incident.place_name
        .toLocaleLowerCase('es-UY')
        .includes(debouncedSearch);
      const descMatch = incident.description
        .toLocaleLowerCase('es-UY')
        .includes(debouncedSearch);
      return placeMatch || descMatch;
    });
  });

  const openCount = $derived(
    filteredIncidents.filter((inc) => inc.status === 'open').length
  );
  const resolvedCount = $derived(
    filteredIncidents.filter((inc) => inc.status === 'resolved').length
  );
  const hasEligibleServices = $derived(
    cleaningService.items.some((service) => service.status !== 'cancelled')
  );

  async function openCreateForm(): Promise<void> {
    formServiceDate = dateFilter.trim() || dateService.selectedDate;
    incidentFormDirty = false;
    await cleaningService.loadByDate(formServiceDate);
    incidentsService.error = null;
    showForm = true;
  }

  function closeForm(): void {
    showForm = false;
    incidentFormDirty = false;
    incidentsService.error = null;
  }

  async function handleFormDateChange(nextDate: string): Promise<void> {
    formServiceDate = nextDate;
    if (nextDate.trim()) {
      await cleaningService.loadByDate(nextDate.trim());
    }
  }

  async function handleCreateIncident(input: IncidentInput): Promise<void> {
    const created = await incidentsService.create(input);
    if (created) {
      closeForm();
      toastTone = 'success';
      toastMessage = `Incidencia registrada en ${created.place_name}.`;
      await loadIncidents();
    } else if (incidentsService.error) {
      toastTone = 'error';
      toastMessage = incidentsService.error;
    }
  }

  async function toggleIncidentStatus(incident: Incident): Promise<void> {
    const nextStatus: IncidentStatus = incident.status === 'open' ? 'resolved' : 'open';
    const updated = await incidentsService.setStatus(incident.id, nextStatus);
    if (updated) {
      toastTone = 'success';
      toastMessage =
        nextStatus === 'resolved'
          ? `Incidencia en ${incident.place_name} marcada como resuelta.`
          : `Incidencia en ${incident.place_name} reabierta.`;
      await loadIncidents();
    } else if (incidentsService.error) {
      toastTone = 'error';
      toastMessage = incidentsService.error;
    }
  }

  async function clearFilters(): Promise<void> {
    statusFilter = 'all';
    typeFilter = 'all';
    dateFilter = '';
    serviceFilter = 'all';
    searchInput = '';
    debouncedSearch = '';
    await loadIncidents();
  }

  function resolvePhotoSrc(photoUrl: string | null): string | null {
    if (!photoUrl) {
      return null;
    }
    if (
      photoUrl.startsWith('http://') ||
      photoUrl.startsWith('https://') ||
      photoUrl.startsWith('asset:') ||
      photoUrl.startsWith('data:')
    ) {
      return photoUrl;
    }
    if (isTauriRuntime()) {
      return convertFileSrc(photoUrl);
    }
    return null;
  }
</script>

<svelte:head>
  <title>Incidencias · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">Seguimiento operativo</p>
    <h1 class="page-title">Incidencias</h1>
    <p class="page-lede">
      Registrá y resolvé daños, objetos olvidados o faltantes de insumos detectados durante o
      después de los servicios de limpieza.
    </p>
  </div>
  {#if !showForm}
    <button type="button" class="button button-primary" onclick={openCreateForm}>
      Registrar incidencia
    </button>
  {/if}
</header>

<Toast message={toastMessage} tone={toastTone} ondismiss={() => (toastMessage = null)} />

{#if showForm}
  <Modal
    open={showForm}
    title="Registrar incidencia"
    description="Asociá la incidencia a un servicio de limpieza y detallá lo ocurrido."
    size="lg"
    formId="incident-form"
    submitLabel="Registrar incidencia"
    submitDisabled={!hasEligibleServices}
    busy={incidentsService.saving}
    dirty={incidentFormDirty}
    onclose={closeForm}
  >
    <div class="form-date-bar">
      <div>
        <strong>Fecha del servicio a asociar</strong>
        <p>Elegí la fecha para listar los servicios de limpieza disponibles.</p>
      </div>
      <input
        class="input date-picker"
        type="date"
        aria-label="Fecha de los servicios disponibles"
        value={formServiceDate}
        onchange={(e) => handleFormDateChange((e.currentTarget as HTMLInputElement).value)}
      />
    </div>

    <IncidentForm
      formId="incident-form"
      services={cleaningService.items}
      defaultDate={formServiceDate}
      busy={incidentsService.saving}
      error={incidentsService.error}
      bind:dirty={incidentFormDirty}
      onsubmit={handleCreateIncident}
    />
  </Modal>
{/if}

<section class="toolbar panel" aria-label="Filtros de incidencias">
  <div class="summary-strip">
    <div class="summary-badges">
      <span class="badge badge-status-open">
        {openCount} abierta{openCount === 1 ? '' : 's'}
      </span>
      <span class="badge badge-status-resolved">
        {resolvedCount} resuelta{resolvedCount === 1 ? '' : 's'}
      </span>
    </div>
    {#if statusFilter !== 'all' || typeFilter !== 'all' || dateFilter || serviceFilter !== 'all' || searchInput}
      <button type="button" class="button button-secondary button-sm" onclick={clearFilters}>
        Limpiar filtros
      </button>
    {/if}
  </div>

  <div class="filters-grid">
    <div class="filter-field filter-search">
      <label for="incident-search">Buscar por lugar o descripción</label>
      <input
        id="incident-search"
        class="input"
        type="search"
        placeholder="Ej. Apto 204, lámpara, control remoto…"
        value={searchInput}
        oninput={handleSearchInput}
      />
    </div>

    <div class="filter-field">
      <label for="incident-filter-status">Estado</label>
      <select
        id="incident-filter-status"
        class="input"
        bind:value={statusFilter}
        onchange={loadIncidents}
      >
        <option value="all">Todos los estados</option>
        {#each incidentStatusOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>

    <div class="filter-field">
      <label for="incident-filter-type">Tipo</label>
      <select
        id="incident-filter-type"
        class="input"
        bind:value={typeFilter}
        onchange={loadIncidents}
      >
        <option value="all">Todos los tipos</option>
        {#each incidentTypeOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>

    <div class="filter-field">
      <label for="incident-filter-date">Fecha</label>
      <input
        id="incident-filter-date"
        class="input"
        type="date"
        bind:value={dateFilter}
        onchange={loadIncidents}
      />
    </div>

    <div class="filter-field">
      <label for="incident-filter-service">Servicio</label>
      <select
        id="incident-filter-service"
        class="input"
        bind:value={serviceFilter}
        onchange={loadIncidents}
      >
        <option value="all">Todos los servicios</option>
        {#each availableServiceFilterOptions as option (option.id)}
          <option value={option.id}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>
</section>

{#if incidentsService.error && !showForm}
  <section class="panel error-banner" role="alert">
    <p>{incidentsService.error}</p>
    <button type="button" class="button button-secondary" onclick={loadIncidents}>
      Reintentar
    </button>
  </section>
{/if}

{#if incidentsService.loading && incidentsService.items.length === 0}
  <section class="panel status-panel" role="status">
    <p>Cargando incidencias…</p>
  </section>
{:else if filteredIncidents.length === 0}
  <section class="panel status-panel">
    <EmptyState
      title="No se encontraron incidencias"
      description={statusFilter !== 'all' ||
      typeFilter !== 'all' ||
      dateFilter ||
      serviceFilter !== 'all' ||
      debouncedSearch
        ? 'Ninguna incidencia coincide con los filtros actuales. Probá ampliar la búsqueda.'
        : 'No hay incidencias registradas todavía. Podés anotar cualquier daño, objeto olvidado o faltante de insumos asociado a un servicio.'}
    >
      {#if !showForm}
        <button type="button" class="button button-primary" onclick={openCreateForm}>
          Registrar incidencia
        </button>
      {/if}
    </EmptyState>
  </section>
{:else}
  <div class="incidents-grid">
    {#each filteredIncidents as incident (incident.id)}
      {@const photoSrc = resolvePhotoSrc(incident.photo_url)}
      <article class={`incident-card panel status-${incident.status}`}>
        <div class="incident-top">
          <div>
            <div class="title-row">
              <h2>{incident.place_name}</h2>
              <span class={`badge badge-type-${incident.type}`}>
                {labelOf(incidentTypeLabels, incident.type)}
              </span>
              <span class="badge badge-service">
                {labelOf(serviceTypeLabels, incident.service_type)}
              </span>
              <span class={`badge badge-status-${incident.status}`}>
                {labelOf(incidentStatusLabels, incident.status)}
              </span>
            </div>
            <p class="meta-line">
              Fecha: {formatDate(incident.date)}
              {#if incident.status === 'resolved' && incident.resolved_at}
                · Resuelta el {formatUtcDateTimeDate(incident.resolved_at)}
              {/if}
            </p>
          </div>
        </div>

        <p class="incident-description">{incident.description}</p>

        {#if photoSrc}
          <div class="incident-photo-box">
            <button
              type="button"
              class="photo-thumb-button"
              onclick={() => (previewIncident = incident)}
              aria-label={`Ampliar foto de la incidencia en ${incident.place_name}`}
            >
              <img
                src={photoSrc}
                alt={`Evidencia en ${incident.place_name}`}
                class="incident-thumb"
              />
              <span class="photo-caption">Ver foto ampliada</span>
            </button>
          </div>
        {/if}

        <div class="card-actions">
          {#if incident.status === 'open'}
            <button
              type="button"
              class="button button-primary button-sm"
              onclick={() => toggleIncidentStatus(incident)}
              disabled={incidentsService.saving}
            >
              Marcar como resuelta
            </button>
          {:else}
            <button
              type="button"
              class="button button-secondary button-sm"
              onclick={() => toggleIncidentStatus(incident)}
              disabled={incidentsService.saving}
            >
              Reabrir incidencia
            </button>
          {/if}
        </div>
      </article>
    {/each}
  </div>
{/if}

<Modal
  open={previewIncident !== null}
  title={previewIncident
    ? `${previewIncident.place_name} · ${labelOf(incidentTypeLabels, previewIncident.type)}`
    : 'Foto de incidencia'}
  description={previewIncident?.description ?? ''}
  size="lg"
  onclose={() => (previewIncident = null)}
>
  {#if previewIncident}
    {@const modalSrc = resolvePhotoSrc(previewIncident.photo_url)}
    {#if modalSrc}
      <div class="photo-modal-body">
        <img
          src={modalSrc}
          alt={`Foto de incidencia en ${previewIncident.place_name}`}
          class="photo-modal-img"
        />
      </div>
    {/if}
  {/if}
</Modal>

<style>
  .form-date-bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    margin-bottom: var(--space-5);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
  }

  .form-date-bar p {
    margin-top: 0.15rem;
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .date-picker {
    width: auto;
  }

  .toolbar {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    margin-bottom: var(--space-5);
  }

  .summary-strip {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-line);
  }

  .summary-badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .filters-grid {
    display: grid;
    grid-template-columns: 1.8fr repeat(4, minmax(10rem, 1fr));
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

  .status-panel {
    padding: var(--space-6);
  }

  .incidents-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(20rem, 100%), 1fr));
    gap: var(--space-3);
  }

  .incident-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-4);
  }

  .incident-card.status-open {
    border-left: 4px solid var(--color-danger);
  }

  .title-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .title-row h2 {
    font-size: var(--text-base);
  }

  .meta-line {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .incident-description {
    color: var(--color-ink);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
    overflow-wrap: anywhere;
    white-space: pre-line;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.7rem;
    font-weight: 650;
  }

  .badge-type-damage {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-type-lost_item {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .badge-type-supply_shortage {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .badge-service {
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
  }

  .badge-status-open {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-status-resolved {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .incident-photo-box {
    display: flex;
    align-items: flex-start;
  }

  .photo-thumb-button {
    display: inline-flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
    text-align: left;
  }

  .photo-thumb-button:hover {
    border-color: var(--color-primary);
  }

  .incident-thumb {
    display: block;
    max-width: 11rem;
    max-height: 7.5rem;
    border-radius: var(--radius-sm);
    object-fit: cover;
  }

  .photo-caption {
    color: var(--color-primary);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .card-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-line);
  }

  .button-sm {
    min-height: 2.1rem;
    padding: 0.4rem 0.65rem;
    font-size: var(--text-xs);
  }

  .photo-modal-body {
    display: flex;
    justify-content: center;
  }

  .photo-modal-img {
    max-width: 100%;
    max-height: 62vh;
    border-radius: var(--radius-md);
    object-fit: contain;
  }

  @media (max-width: 1100px) {
    .filters-grid {
      grid-template-columns: 1fr 1fr;
    }

    .filter-search {
      grid-column: 1 / -1;
    }
  }

  @media (max-width: 640px) {
    .filters-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
