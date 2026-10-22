<script lang="ts">
  import { onMount } from 'svelte';

  import type {
    CleaningService,
    CleaningServiceInput,
    Incident,
    IncidentInput,
    Place,
    PlaceStatus,
    ServiceType
  } from '#lib/api/types.js';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import DailyBoard from '#lib/components/DailyBoard.svelte';
  import IncidentForm from '#lib/components/IncidentForm.svelte';
  import ServiceForm from '#lib/components/ServiceForm.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { cleaningService } from '#lib/services/cleaningService.svelte.js';
  import { dateService } from '#lib/services/dateService.svelte.js';
  import { incidentsService } from '#lib/services/incidentsService.svelte.js';
  import { placesService } from '#lib/services/placesService.svelte.js';
  import { staffService } from '#lib/services/staffService.svelte.js';
  import { nowTimeIso } from '#lib/utils/format.js';

  let showServiceForm = $state(false);
  let editingService = $state<CleaningService | null>(null);
  let presetPlaceId = $state<number | null>(null);
  let presetServiceType = $state<ServiceType | null>(null);

  let showIncidentForm = $state(false);
  let presetIncidentServiceId = $state<number | null>(null);

  let completingService = $state<CleaningService | null>(null);
  let completeStartTime = $state('09:00');
  let completeEndTime = $state('10:00');

  let cancellingService = $state<CleaningService | null>(null);

  let toastMessage = $state<string | null>(null);
  let toastTone = $state<'success' | 'warning' | 'error'>('success');

  const isLoading = $derived(
    cleaningService.loading ||
      placesService.loading ||
      staffService.loading ||
      incidentsService.loading
  );

  const isBusy = $derived(
    cleaningService.saving ||
      placesService.saving ||
      staffService.saving ||
      incidentsService.saving
  );

  const loadError = $derived(
    cleaningService.error ??
      placesService.error ??
      staffService.error ??
      incidentsService.error
  );

  async function loadBoardForDate(date: string): Promise<void> {
    await Promise.all([
      placesService.load(date),
      cleaningService.loadByDate(date),
      staffService.load(),
      incidentsService.load({ status: 'open' })
    ]);
  }

  onMount(() => {
    void loadBoardForDate(dateService.selectedDate);
  });

  async function refreshDateData(date: string): Promise<void> {
    await Promise.all([
      placesService.load(date),
      cleaningService.loadByDate(date)
    ]);
  }

  async function handlePrevDay(): Promise<void> {
    dateService.previousDay();
    await refreshDateData(dateService.selectedDate);
  }

  async function handleNextDay(): Promise<void> {
    dateService.nextDay();
    await refreshDateData(dateService.selectedDate);
  }

  async function handleToday(): Promise<void> {
    dateService.goToToday();
    await refreshDateData(dateService.selectedDate);
  }

  async function handleDateChange(nextDate: string): Promise<void> {
    if (!nextDate.trim()) {
      return;
    }
    dateService.setDate(nextDate);
    await refreshDateData(dateService.selectedDate);
  }

  function openCreateService(preset?: { placeId: number; serviceType: ServiceType }): void {
    editingService = null;
    presetPlaceId = preset?.placeId ?? null;
    presetServiceType = preset?.serviceType ?? null;
    showIncidentForm = false;
    completingService = null;
    cleaningService.error = null;
    showServiceForm = true;
  }

  function openEditService(service: CleaningService): void {
    editingService = service;
    presetPlaceId = null;
    presetServiceType = null;
    showIncidentForm = false;
    completingService = null;
    cleaningService.error = null;
    showServiceForm = true;
  }

  function closeServiceForm(): void {
    showServiceForm = false;
    editingService = null;
    presetPlaceId = null;
    presetServiceType = null;
    cleaningService.error = null;
  }

  function openAddIncident(serviceId?: number): void {
    presetIncidentServiceId = serviceId ?? null;
    showServiceForm = false;
    completingService = null;
    incidentsService.error = null;
    showIncidentForm = true;
  }

  function closeIncidentForm(): void {
    showIncidentForm = false;
    presetIncidentServiceId = null;
    incidentsService.error = null;
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      if (completingService) {
        completingService = null;
      } else if (showIncidentForm) {
        closeIncidentForm();
      } else if (showServiceForm) {
        closeServiceForm();
      }
    }
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

  async function handleServiceSubmit(payload: {
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
          await placesService.load(dateService.selectedDate);
        }
        closeServiceForm();
      }
    } else {
      const created = await cleaningService.create(payload.input);
      if (created) {
        if (cleaningService.lastWarning) {
          toastTone = 'warning';
          toastMessage = cleaningService.lastWarning;
        } else {
          toastTone = 'success';
          toastMessage = `Servicio programado en ${created.place_name}.`;
        }
        if (payload.input.date !== dateService.selectedDate) {
          dateService.setDate(payload.input.date);
          await placesService.load(dateService.selectedDate);
        }
        closeServiceForm();
      }
    }
  }

  async function handleStartService(service: CleaningService): Promise<void> {
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
    showServiceForm = false;
    showIncidentForm = false;
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

  async function handleTogglePlaceCancelled(place: Place): Promise<void> {
    const nextStatus: PlaceStatus | null = place.status === 'cancelled' ? null : 'cancelled';
    const updated = await placesService.setStatusOverride(place, nextStatus);
    if (updated) {
      toastTone = 'success';
      toastMessage =
        nextStatus === 'cancelled'
          ? `Turno de limpieza omitido para ${place.name}.`
          : `Recomendación automática restaurada para ${place.name}.`;
    } else if (placesService.error) {
      toastTone = 'error';
      toastMessage = placesService.error;
    }
  }

  async function handleIncidentSubmit(input: IncidentInput): Promise<void> {
    const created = await incidentsService.create(input);
    if (created) {
      toastTone = 'success';
      toastMessage = `Incidencia registrada en ${created.place_name}.`;
      closeIncidentForm();
    }
  }

  async function handleResolveIncident(incident: Incident): Promise<void> {
    const resolved = await incidentsService.setStatus(incident.id, 'resolved');
    if (resolved) {
      toastTone = 'success';
      toastMessage = `Incidencia en ${incident.place_name} marcada como resuelta.`;
    } else if (incidentsService.error) {
      toastTone = 'error';
      toastMessage = incidentsService.error;
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<svelte:head>
  <title>Tablero · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">{dateService.formattedSelectedDate}</p>
    <h1 class="page-title">Tablero del día</h1>
    <p class="page-lede">Qué hay que limpiar hoy, quién está disponible y qué queda pendiente.</p>
  </div>

  <div class="header-actions">
    {#if !showIncidentForm}
      <button
        type="button"
        class="button button-secondary"
        onclick={() => openAddIncident()}
        disabled={isBusy}
      >
        Registrar incidencia
      </button>
    {/if}
    {#if !showServiceForm}
      <button
        type="button"
        class="button button-primary"
        onclick={() => openCreateService()}
        disabled={isBusy}
      >
        Nuevo servicio
      </button>
    {/if}
  </div>
</header>

<Toast message={toastMessage} tone={toastTone} ondismiss={() => (toastMessage = null)} />

{#if loadError && !showServiceForm && !showIncidentForm}
  <div class="error-banner panel" role="alert">
    <p>{loadError}</p>
    <button
      type="button"
      class="button button-secondary"
      onclick={() => loadBoardForDate(dateService.selectedDate)}
    >
      Reintentar
    </button>
  </div>
{/if}

{#if showServiceForm}
  <ServiceForm
    initial={editingService}
    initialPlaceId={presetPlaceId}
    initialServiceType={presetServiceType}
    defaultDate={dateService.selectedDate}
    places={placesService.items}
    allStaff={staffService.items}
    suggestions={cleaningService.suggestions}
    loadingSuggestions={cleaningService.loadingSuggestions}
    busy={cleaningService.saving}
    error={cleaningService.error}
    onrequestsuggestions={handleRequestSuggestions}
    onsubmit={handleServiceSubmit}
    oncancel={closeServiceForm}
  />
{/if}

{#if showIncidentForm}
  <IncidentForm
    services={cleaningService.items}
    initialServiceId={presetIncidentServiceId}
    defaultDate={dateService.selectedDate}
    busy={incidentsService.saving}
    error={incidentsService.error}
    onsubmit={handleIncidentSubmit}
    oncancel={closeIncidentForm}
  />
{/if}

{#if completingService}
  <section class="complete-panel panel" aria-label="Completar servicio">
    <form class="complete-form" onsubmit={submitCompletePrompt}>
      <div>
        <h2>Completar servicio en {completingService.place_name}</h2>
        <p>Confirmá la hora de inicio y de finalización para registrar la duración real.</p>
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

<DailyBoard
  selectedDate={dateService.selectedDate}
  isToday={dateService.isToday}
  places={placesService.items}
  services={cleaningService.items}
  staff={staffService.items}
  absences={staffService.absences}
  openIncidents={incidentsService.openItems}
  loading={isLoading}
  busy={isBusy}
  onprevday={handlePrevDay}
  onnextday={handleNextDay}
  ontoday={handleToday}
  ondatechange={handleDateChange}
  oncreateservice={openCreateService}
  oneditservice={openEditService}
  onstartservice={handleStartService}
  oncompleteservice={openCompletePrompt}
  oncancelservice={(service) => (cancellingService = service)}
  ontoggleplacecancelled={handleTogglePlaceCancelled}
  onaddincident={openAddIncident}
  onresolveincident={handleResolveIncident}
/>

<ConfirmDialog
  open={cancellingService !== null}
  title="Cancelar servicio de limpieza"
  description={cancellingService
    ? `¿Querés cancelar el servicio programado en «${cancellingService.place_name}»? Los servicios cancelados se excluyen de reportes y promedios.`
    : ''}
  confirmLabel="Cancelar servicio"
  tone="danger"
  busy={cleaningService.saving}
  onconfirm={confirmCancelService}
  oncancel={() => (cancellingService = null)}
/>

<style>
  .header-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    margin-bottom: var(--space-5);
    border-color: var(--color-danger);
    background: var(--color-danger-soft);
    font-size: var(--text-sm);
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
</style>
