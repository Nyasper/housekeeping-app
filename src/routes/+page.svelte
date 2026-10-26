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
  import CompleteServiceForm from '#lib/components/CompleteServiceForm.svelte';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import DailyBoard from '#lib/components/DailyBoard.svelte';
  import IncidentForm from '#lib/components/IncidentForm.svelte';
  import Modal from '#lib/components/Modal.svelte';
  import ServiceForm from '#lib/components/ServiceForm.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { cleaningService } from '#lib/services/cleaningService.svelte.js';
  import { dateService } from '#lib/services/dateService.svelte.js';
  import { incidentsService } from '#lib/services/incidentsService.svelte.js';
  import { placesService } from '#lib/services/placesService.svelte.js';
  import { staffService } from '#lib/services/staffService.svelte.js';
  import { nowTimeIso } from '#lib/utils/format.js';

  let showServiceForm = $state(false);
  let serviceFormDirty = $state(false);
  let editingService = $state<CleaningService | null>(null);
  let presetPlaceId = $state<number | null>(null);
  let presetServiceType = $state<ServiceType | null>(null);

  let showIncidentForm = $state(false);
  let incidentFormDirty = $state(false);
  let presetIncidentServiceId = $state<number | null>(null);

  let completingService = $state<CleaningService | null>(null);
  let completeDirty = $state(false);
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

  const serviceFormHasPlaces = $derived(
    placesService.items.some((place) => place.is_active || place.id === editingService?.place_id)
  );

  const incidentFormHasServices = $derived(
    cleaningService.items.some((service) => service.status !== 'cancelled')
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

  function handleDateInput(event: Event): void {
    const input = event.currentTarget as HTMLInputElement;
    if (!input.value.trim()) {
      input.value = dateService.selectedDate;
      return;
    }
    void handleDateChange(input.value);
  }

  function openCreateService(preset?: { placeId: number; serviceType: ServiceType }): void {
    editingService = null;
    serviceFormDirty = false;
    presetPlaceId = preset?.placeId ?? null;
    presetServiceType = preset?.serviceType ?? null;
    showIncidentForm = false;
    incidentFormDirty = false;
    completingService = null;
    cleaningService.error = null;
    cleaningService.resetSuggestions();
    showServiceForm = true;
  }

  function openEditService(service: CleaningService): void {
    editingService = service;
    serviceFormDirty = false;
    presetPlaceId = null;
    presetServiceType = null;
    showIncidentForm = false;
    incidentFormDirty = false;
    completingService = null;
    cleaningService.error = null;
    cleaningService.resetSuggestions();
    showServiceForm = true;
  }

  function closeServiceForm(): void {
    showServiceForm = false;
    serviceFormDirty = false;
    editingService = null;
    presetPlaceId = null;
    presetServiceType = null;
    cleaningService.error = null;
  }

  function openAddIncident(serviceId?: number): void {
    presetIncidentServiceId = serviceId ?? null;
    incidentFormDirty = false;
    showServiceForm = false;
    serviceFormDirty = false;
    completingService = null;
    incidentsService.error = null;
    showIncidentForm = true;
  }

  function closeIncidentForm(): void {
    showIncidentForm = false;
    incidentFormDirty = false;
    presetIncidentServiceId = null;
    incidentsService.error = null;
  }

  function handleRequestSuggestions(params: {
    placeId: number;
    serviceType: ServiceType;
    date: string;
    startTime: string | null;
    endTime: string | null;
    excludeServiceId: number | null;
  }): void {
    void cleaningService.suggestStaff(
      params.placeId,
      params.serviceType,
      params.date,
      params.startTime,
      params.endTime,
      10,
      params.excludeServiceId
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
    const endTime = service.end_time && service.end_time <= start ? null : undefined;
    const updated = await cleaningService.setStatus(service, 'in_progress', start, endTime);
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
    cleaningService.error = null;
    completingService = service;
    completeDirty = false;
    completeStartTime = service.start_time ?? '09:00';
    const current = nowTimeIso();
    completeEndTime =
      service.end_time ?? (current > completeStartTime ? current : '10:00');
  }

  function closeCompletePrompt(): void {
    completingService = null;
    completeDirty = false;
  }

  async function submitCompletePrompt({
    startTime,
    endTime
  }: {
    startTime: string;
    endTime: string;
  }): Promise<void> {
    const target = completingService;
    if (!target) {
      return;
    }
    const updated = await cleaningService.setStatus(target, 'completed', startTime, endTime);
    if (updated) {
      closeCompletePrompt();
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
  <Modal
    open={showServiceForm}
    title={editingService ? 'Editar servicio de limpieza' : 'Nuevo servicio de limpieza'}
    description="Seleccioná el lugar, el tipo de tarea, el horario y al menos un funcionario disponible."
    size="xl"
    formId="board-service-form"
    submitLabel={editingService ? 'Guardar cambios' : 'Crear servicio'}
    submitDisabled={!serviceFormHasPlaces}
    busy={cleaningService.saving}
    dirty={serviceFormDirty}
    onclose={closeServiceForm}
  >
    <ServiceForm
      formId="board-service-form"
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
      bind:dirty={serviceFormDirty}
      onrequestsuggestions={handleRequestSuggestions}
      onsubmit={handleServiceSubmit}
    />
  </Modal>
{/if}

{#if showIncidentForm}
  <Modal
    open={showIncidentForm}
    title="Registrar incidencia"
    description="Anotá daños, objetos olvidados o faltantes de insumos detectados durante o después de una limpieza."
    size="lg"
    formId="board-incident-form"
    submitLabel="Registrar incidencia"
    submitDisabled={!incidentFormHasServices}
    busy={incidentsService.saving}
    dirty={incidentFormDirty}
    onclose={closeIncidentForm}
  >
    <IncidentForm
      formId="board-incident-form"
      services={cleaningService.items}
      initialServiceId={presetIncidentServiceId}
      defaultDate={dateService.selectedDate}
      busy={incidentsService.saving}
      error={incidentsService.error}
      bind:dirty={incidentFormDirty}
      onsubmit={handleIncidentSubmit}
    />
  </Modal>
{/if}

<Modal
  open={completingService !== null}
  title={`Completar servicio en ${completingService?.place_name ?? ''}`}
  description="Confirmá la hora de inicio y de finalización para registrar la duración real."
  size="sm"
  formId="board-complete-service-form"
  submitLabel="Confirmar completado"
  busy={cleaningService.saving}
  dirty={completeDirty}
  onclose={closeCompletePrompt}
>
  {#if completingService}
    <CompleteServiceForm
      formId="board-complete-service-form"
      initialStartTime={completeStartTime}
      initialEndTime={completeEndTime}
      error={cleaningService.error}
      bind:dirty={completeDirty}
      onsubmit={submitCompletePrompt}
    />
  {/if}
</Modal>

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
</style>
