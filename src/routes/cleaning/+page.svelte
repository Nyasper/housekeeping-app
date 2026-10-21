<script lang="ts">
  import { onMount } from 'svelte';

  import {
    labelOf,
    serviceStatusLabels,
    serviceTypeLabels,
    toOptions
  } from '#lib/api/labels.js';
  import type {
    CleaningService,
    CleaningServiceInput,
    ServiceStatus,
    ServiceType
  } from '#lib/api/types.js';
  import Avatar from '#lib/components/Avatar.svelte';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import EmptyState from '#lib/components/EmptyState.svelte';
  import ServiceForm from '#lib/components/ServiceForm.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { cleaningService } from '#lib/services/cleaningService.svelte.js';
  import { dateService } from '#lib/services/dateService.svelte.js';
  import { placesService } from '#lib/services/placesService.svelte.js';
  import { staffService } from '#lib/services/staffService.svelte.js';
  import {
    durationMinutesBetween,
    formatDurationMinutes,
    formatTime,
    nowTimeIso
  } from '#lib/utils/format.js';

  const serviceTypeOptions = toOptions(serviceTypeLabels);
  const serviceStatusOptions = toOptions(serviceStatusLabels);

  let searchInput = $state('');
  let debouncedSearch = $state('');
  let statusFilter = $state<ServiceStatus | 'all'>('all');
  let typeFilter = $state<ServiceType | 'all'>('all');

  let showForm = $state(false);
  let editingService = $state<CleaningService | null>(null);
  let cancellingService = $state<CleaningService | null>(null);
  let completingService = $state<CleaningService | null>(null);
  let completeStartTime = $state('09:00');
  let completeEndTime = $state('10:00');

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

  async function loadAllForSelectedDate(): Promise<void> {
    await Promise.all([
      cleaningService.loadByDate(dateService.selectedDate),
      placesService.load(),
      staffService.load()
    ]);
  }

  onMount(() => {
    void loadAllForSelectedDate();
  });

  async function changeDate(nextDate: string): Promise<void> {
    dateService.setDate(nextDate);
    await cleaningService.loadByDate(dateService.selectedDate);
  }

  async function goPrevDay(): Promise<void> {
    dateService.previousDay();
    await cleaningService.loadByDate(dateService.selectedDate);
  }

  async function goNextDay(): Promise<void> {
    dateService.nextDay();
    await cleaningService.loadByDate(dateService.selectedDate);
  }

  async function goToday(): Promise<void> {
    dateService.goToToday();
    await cleaningService.loadByDate(dateService.selectedDate);
  }

  const activeStaff = $derived(staffService.items.filter((m) => m.is_active));

  const filteredServices = $derived.by(() => {
    return cleaningService.items.filter((service) => {
      if (statusFilter !== 'all' && service.status !== statusFilter) {
        return false;
      }
      if (typeFilter !== 'all' && service.service_type !== typeFilter) {
        return false;
      }
      if (debouncedSearch) {
        const placeMatch = service.place_name.toLocaleLowerCase('es-UY').includes(debouncedSearch);
        const staffMatch = service.staff.some((s) =>
          `${s.first_name} ${s.last_name}`.toLocaleLowerCase('es-UY').includes(debouncedSearch)
        );
        if (!placeMatch && !staffMatch) {
          return false;
        }
      }
      return true;
    });
  });

  function openCreateForm(): void {
    editingService = null;
    showForm = true;
    cleaningService.error = null;
  }

  function openEditForm(service: CleaningService): void {
    editingService = service;
    showForm = true;
    cleaningService.error = null;
  }

  function closeForm(): void {
    showForm = false;
    editingService = null;
    cleaningService.error = null;
  }

  function handleRequestSuggestions(params: {
    placeId: number;
    serviceType: ServiceType;
    date: string;
    startTime: string | null;
    endTime: string | null;
  }): void {
    void cleaningService.suggestStaff(
      params.placeId,
      params.serviceType,
      params.date,
      params.startTime,
      params.endTime
    );
  }

  async function handleFormSubmit(payload: {
    input: CleaningServiceInput;
    confirmReopen: boolean;
  }): Promise<void> {
    if (editingService) {
      const updated = await cleaningService.update(
        editingService.id,
        payload.input,
        payload.confirmReopen
      );
      if (updated) {
        if (cleaningService.lastWarning) {
          toastTone = 'warning';
          toastMessage = cleaningService.lastWarning;
        } else {
          toastTone = 'success';
          toastMessage = 'Servicio actualizado.';
        }
        if (payload.input.date !== dateService.selectedDate) {
          dateService.setDate(payload.input.date);
        }
        closeForm();
      }
    } else {
      const created = await cleaningService.create(payload.input);
      if (created) {
        if (cleaningService.lastWarning) {
          toastTone = 'warning';
          toastMessage = cleaningService.lastWarning;
        } else {
          toastTone = 'success';
          toastMessage = 'Servicio creado.';
        }
        if (payload.input.date !== dateService.selectedDate) {
          dateService.setDate(payload.input.date);
        }
        closeForm();
      }
    }
  }

  async function startServiceNow(service: CleaningService): Promise<void> {
    const start = service.start_time ?? nowTimeIso();
    const updated = await cleaningService.setStatus(service, 'in_progress', start, null);
    if (updated) {
      toastTone = 'success';
      toastMessage = `Servicio en ${service.place_name} iniciado (${start}).`;
    } else if (cleaningService.error) {
      toastTone = 'error';
      toastMessage = cleaningService.error;
    }
  }

  function openCompletePrompt(service: CleaningService): void {
    completingService = service;
    completeStartTime = service.start_time ?? '09:00';
    const current = nowTimeIso();
    completeEndTime =
      service.end_time ?? (current > completeStartTime ? current : '10:00');
  }

  async function submitCompletePrompt(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const target = completingService;
    if (!target) {
      return;
    }
    const updated = await cleaningService.setStatus(
      target,
      'completed',
      completeStartTime,
      completeEndTime
    );
    if (updated) {
      completingService = null;
      toastTone = 'success';
      toastMessage = `Servicio en ${target.place_name} completado.`;
    } else if (cleaningService.error) {
      toastTone = 'error';
      toastMessage = cleaningService.error;
    }
  }

  async function confirmCancelService(): Promise<void> {
    const target = cancellingService;
    if (!target) {
      return;
    }
    const updated = await cleaningService.setStatus(target, 'cancelled');
    cancellingService = null;
    if (updated) {
      toastTone = 'success';
      toastMessage = `Servicio en ${target.place_name} cancelado.`;
    } else if (cleaningService.error) {
      toastTone = 'error';
      toastMessage = cleaningService.error;
    }
  }
</script>

<svelte:head>
  <title>Servicios · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">{dateService.formattedSelectedDate}</p>
    <h1 class="page-title">Servicios de limpieza</h1>
    <p class="page-lede">
      Planificá, asigná personal con asistencia y registrá los horarios de inicio y fin de cada
      limpieza.
    </p>
  </div>
  {#if !showForm}
    <button type="button" class="button button-primary" onclick={openCreateForm}>
      Nuevo servicio
    </button>
  {/if}
</header>

<Toast message={toastMessage} tone={toastTone} ondismiss={() => (toastMessage = null)} />

{#if showForm}
  <ServiceForm
    initial={editingService}
    defaultDate={dateService.selectedDate}
    places={placesService.items}
    allActiveStaff={activeStaff}
    suggestions={cleaningService.suggestions}
    loadingSuggestions={cleaningService.loadingSuggestions}
    busy={cleaningService.saving}
    error={cleaningService.error}
    onrequestsuggestions={handleRequestSuggestions}
    onsubmit={handleFormSubmit}
    oncancel={closeForm}
  />
{/if}

{#if completingService}
  <section class="complete-panel panel" aria-label="Completar servicio">
    <form class="complete-form" onsubmit={submitCompletePrompt}>
      <div>
        <h2>Completar servicio en {completingService.place_name}</h2>
        <p>Confirmá la hora de inicio y de finalización para calcular la duración del servicio.</p>
      </div>
      <div class="complete-times">
        <label>
          <span>Hora de inicio</span>
          <input class="input" type="time" bind:value={completeStartTime} required />
        </label>
        <label>
          <span>Hora de fin</span>
          <input class="input" type="time" bind:value={completeEndTime} required />
        </label>
      </div>
      <div class="complete-actions">
        <button type="submit" class="button button-primary" disabled={cleaningService.saving}>
          Confirmar completado
        </button>
        <button
          type="button"
          class="button button-secondary"
          onclick={() => (completingService = null)}
        >
          Cancelar
        </button>
      </div>
    </form>
  </section>
{/if}

<section class="toolbar panel" aria-label="Fecha y filtros de servicios">
  <div class="date-controls">
    <button type="button" class="button button-secondary" onclick={goPrevDay}>
      ← Día anterior
    </button>
    <input
      class="input date-picker"
      type="date"
      aria-label="Fecha de los servicios"
      value={dateService.selectedDate}
      onchange={(e) => changeDate((e.currentTarget as HTMLInputElement).value)}
    />
    <button type="button" class="button button-secondary" onclick={goNextDay}>
      Día siguiente →
    </button>
    {#if !dateService.isToday}
      <button type="button" class="button button-secondary" onclick={goToday}>
        Hoy
      </button>
    {/if}
  </div>

  <div class="filters-row">
    <div class="filter-field filter-search">
      <label for="service-search">Buscar por lugar o funcionario</label>
      <input
        id="service-search"
        class="input"
        type="search"
        placeholder="Ej. Apto 101 o María"
        value={searchInput}
        oninput={handleSearchInput}
      />
    </div>

    <div class="filter-field">
      <label for="service-filter-status">Estado</label>
      <select id="service-filter-status" class="input" bind:value={statusFilter}>
        <option value="all">Todos los estados</option>
        {#each serviceStatusOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>

    <div class="filter-field">
      <label for="service-filter-type">Tipo de servicio</label>
      <select id="service-filter-type" class="input" bind:value={typeFilter}>
        <option value="all">Todos los tipos</option>
        {#each serviceTypeOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>
</section>

{#if cleaningService.loading && cleaningService.items.length === 0}
  <section class="panel status-panel" role="status">
    <p>Cargando servicios del día…</p>
  </section>
{:else if filteredServices.length === 0}
  <section class="panel status-panel">
    <EmptyState
      title="No hay servicios para esta fecha"
      description="Creá un servicio de limpieza seleccionando el apartamento o área común y el personal sugerido."
    >
      {#if !showForm}
        <button type="button" class="button button-primary" onclick={openCreateForm}>
          Crear servicio
        </button>
      {/if}
    </EmptyState>
  </section>
{:else}
  <div class="services-grid">
    {#each filteredServices as service (service.id)}
      {@const duration = durationMinutesBetween(service.start_time, service.end_time)}
      {@const hasInactiveStaff = service.staff.some((s) => s.is_active === false)}
      <article class={`service-card panel ${hasInactiveStaff ? 'has-anomaly' : ''}`}>
        <div class="service-header">
          <div>
            <div class="title-row">
              <h2>{service.place_name}</h2>
              <span class="badge badge-type">
                {labelOf(serviceTypeLabels, service.service_type)}
              </span>
              <span class={`badge badge-status-${service.status}`}>
                {labelOf(serviceStatusLabels, service.status)}
              </span>
              {#if hasInactiveStaff}
                <span class="badge badge-anomaly">Revisar: personal dado de baja</span>
              {/if}
            </div>
            <p class="time-line">
              {#if service.start_time && service.end_time}
                {formatTime(service.start_time)} a {formatTime(service.end_time)}
                {#if duration !== null}
                  · Duración: {formatDurationMinutes(duration)}
                {/if}
              {:else if service.start_time}
                Inicio: {formatTime(service.start_time)} · En curso
              {:else}
                Horario sin definir
              {/if}
            </p>
          </div>
        </div>

        <div class="assigned-staff">
          {#each service.staff as assigned (assigned.staff_id)}
            <div class={`staff-chip ${assigned.is_active === false ? 'is-inactive' : ''}`}>
              <Avatar
                photoUrl={assigned.photo_url}
                gender={assigned.gender}
                alt={`Foto de ${assigned.first_name} ${assigned.last_name}`}
                size="sm"
              />
              <span>{assigned.first_name} {assigned.last_name}</span>
              {#if assigned.is_lead}
                <span class="lead-badge">Responsable</span>
              {/if}
            </div>
          {/each}
        </div>

        {#if service.notes}
          <p class="service-notes">{service.notes}</p>
        {/if}

        <div class="card-actions">
          {#if service.status === 'planned'}
            <button
              type="button"
              class="button button-primary"
              onclick={() => startServiceNow(service)}
              disabled={cleaningService.saving}
            >
              Iniciar
            </button>
          {/if}

          {#if service.status === 'planned' || service.status === 'in_progress'}
            <button
              type="button"
              class="button button-primary"
              onclick={() => openCompletePrompt(service)}
              disabled={cleaningService.saving}
            >
              Completar
            </button>
          {/if}

          <button
            type="button"
            class="button button-secondary"
            onclick={() => openEditForm(service)}
          >
            {service.status === 'completed' ? 'Editar / Reabrir' : 'Editar'}
          </button>

          {#if service.status === 'planned' || service.status === 'in_progress'}
            <button
              type="button"
              class="button button-secondary"
              onclick={() => (cancellingService = service)}
            >
              Cancelar servicio
            </button>
          {/if}
        </div>
      </article>
    {/each}
  </div>
{/if}

<ConfirmDialog
  open={cancellingService !== null}
  title="Cancelar servicio de limpieza"
  description={cancellingService
    ? `¿Querés cancelar el servicio planificado para «${cancellingService.place_name}»? Los servicios cancelados se excluyen de reportes y promedios.`
    : ''}
  confirmLabel="Cancelar servicio"
  tone="danger"
  busy={cleaningService.saving}
  onconfirm={confirmCancelService}
  oncancel={() => (cancellingService = null)}
/>

<style>
  .toolbar {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    margin-bottom: var(--space-5);
  }

  .date-controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-line);
  }

  .date-picker {
    width: auto;
  }

  .filters-row {
    display: grid;
    grid-template-columns: 2fr repeat(2, minmax(11rem, 1fr));
    gap: var(--space-4);
  }

  .filter-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .filter-field label {
    font-size: var(--text-xs);
    font-weight: 650;
    color: var(--color-ink-soft);
  }

  .complete-panel {
    padding: var(--space-5);
    margin-bottom: var(--space-5);
    border-color: var(--color-primary);
  }

  .complete-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .complete-form h2 {
    font-size: var(--text-lg);
  }

  .complete-form p {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .complete-times {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
  }

  .complete-times label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .complete-actions {
    display: flex;
    gap: var(--space-2);
  }

  .status-panel {
    padding: var(--space-6);
  }

  .services-grid {
    display: grid;
    gap: var(--space-4);
  }

  .service-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .service-card.has-anomaly {
    border-color: var(--color-warning);
  }

  .title-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .title-row h2 {
    font-size: var(--text-lg);
  }

  .time-line {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.65rem;
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .badge-type {
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
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

  .badge-anomaly {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .assigned-staff {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .staff-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.3rem 0.7rem 0.3rem 0.35rem;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-pill);
    background: var(--color-surface-raised);
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .staff-chip.is-inactive {
    border-color: var(--color-warning);
    background: var(--color-warning-soft);
  }

  .lead-badge {
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-size: 0.7rem;
    font-weight: 650;
  }

  .service-notes {
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
  }

  .card-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-line);
  }

  @media (max-width: 900px) {
    .filters-row {
      grid-template-columns: 1fr;
    }
  }
</style>
