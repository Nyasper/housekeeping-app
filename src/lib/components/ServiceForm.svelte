<script lang="ts">
  import { untrack } from 'svelte';

  import {
    labelOf,
    serviceStatusLabels,
    serviceTypeLabels,
    toOptions
  } from '#lib/api/labels.js';
  import type {
    CleaningService,
    CleaningServiceInput,
    Place,
    ServiceStaffInput,
    ServiceStatus,
    ServiceType,
    Staff,
    StaffSuggestion
  } from '#lib/api/types.js';
  import AvailabilityList from './AvailabilityList.svelte';
  import FormField from './FormField.svelte';

  interface Props {
    formId: string;
    initial?: CleaningService | null;
    initialPlaceId?: number | null;
    initialServiceType?: ServiceType | null;
    defaultDate: string;
    places: Place[];
    allStaff: Staff[];
    suggestions: StaffSuggestion[];
    loadingSuggestions?: boolean;
    busy?: boolean;
    error?: string | null;
    dirty?: boolean;
    onrequestsuggestions: (params: {
      placeId: number;
      serviceType: ServiceType;
      date: string;
      startTime: string | null;
      endTime: string | null;
      excludeServiceId: number | null;
    }) => void;
    onsubmit: (payload: {
      input: CleaningServiceInput;
      confirmReopen: boolean;
    }) => Promise<void> | void;
  }

  let {
    formId,
    initial = null,
    initialPlaceId = null,
    initialServiceType = null,
    defaultDate,
    places,
    allStaff,
    suggestions,
    loadingSuggestions = false,
    busy = false,
    error = null,
    dirty = $bindable(false),
    onrequestsuggestions,
    onsubmit
  }: Props = $props();

  const serviceTypeOptions = toOptions(serviceTypeLabels);

  const activePlaces = $derived(places.filter((p) => p.is_active || p.id === initial?.place_id));

  let placeId = $derived<number>(
    initial?.place_id ?? initialPlaceId ?? activePlaces[0]?.id ?? 0
  );
  let date = $derived(initial?.date ?? defaultDate);
  let serviceType = $derived<ServiceType>(
    initial?.service_type ??
      initialServiceType ??
      activePlaces.find((p) => p.id === (initialPlaceId ?? activePlaces[0]?.id))
        ?.recommended_service_type ??
      'daily_cleaning'
  );
  let status = $derived<ServiceStatus>(initial?.status ?? 'planned');
  let startTime = $derived(initial?.start_time ?? '');
  let endTime = $derived(initial?.end_time ?? '');
  let notes = $derived(initial?.notes ?? '');
  let selectedStaff = $derived<ServiceStaffInput[]>(
    initial
      ? initial.staff.map((s) => ({ staff_id: s.staff_id, is_lead: s.is_lead }))
      : []
  );
  let confirmReopen = $derived(Boolean(initial && false));
  let submitted = $state(false);

  function staffSignature(items: ServiceStaffInput[]): string {
    return items.map((item) => `${item.staff_id}:${item.is_lead ? 1 : 0}`).join('|');
  }

  const baseline = untrack(() => ({
    placeId,
    date,
    serviceType,
    status,
    startTime,
    endTime,
    notes,
    staff: staffSignature(selectedStaff),
    confirmReopen
  }));

  const isDirty = $derived(
    placeId !== baseline.placeId ||
      date !== baseline.date ||
      serviceType !== baseline.serviceType ||
      status !== baseline.status ||
      startTime !== baseline.startTime ||
      endTime !== baseline.endTime ||
      notes !== baseline.notes ||
      staffSignature(selectedStaff) !== baseline.staff ||
      confirmReopen !== baseline.confirmReopen
  );

  $effect(() => {
    dirty = isDirty;
  });

  const selectedPlace = $derived(places.find((p) => p.id === placeId) ?? null);
  const isCompletedService = $derived(initial?.status === 'completed');
  const isCancelledService = $derived(initial?.status === 'cancelled');

  const serviceStatusOptions = $derived.by(() => {
    const options = toOptions(serviceStatusLabels);
    if (isCompletedService) {
      return options.filter((option) => option.value !== 'cancelled');
    }
    if (isCancelledService) {
      return options.filter(
        (option) => option.value === 'cancelled' || option.value === 'planned'
      );
    }
    return options;
  });

  const showReopenBox = $derived(
    isCompletedService || (isCancelledService && status !== 'cancelled')
  );

  const placeError = $derived(
    submitted && !placeId ? 'Seleccioná el apartamento o área común.' : null
  );

  const staffError = $derived.by(() => {
    if (!submitted) {
      return null;
    }
    if (selectedStaff.length === 0) {
      return 'Debés asignar al menos un funcionario al servicio.';
    }
    const leadCount = selectedStaff.filter((s) => s.is_lead).length;
    if (leadCount > 1) {
      return 'Solo puede haber un responsable principal.';
    }
    return null;
  });

  const timeError = $derived.by(() => {
    if (!submitted) {
      return null;
    }
    const st = startTime.trim();
    const et = endTime.trim();
    if (status === 'in_progress' && !st) {
      return 'Indicá la hora de inicio para marcar el servicio en progreso.';
    }
    if (status === 'completed' && (!st || !et)) {
      return 'Indicá la hora de inicio y la de finalización para completar el servicio.';
    }
    if (st && et && st >= et) {
      return 'La hora de inicio debe ser anterior a la hora de finalización.';
    }
    return null;
  });

  const reopenError = $derived(
    submitted && showReopenBox && !confirmReopen
      ? isCancelledService
        ? 'Confirmá que querés reabrir este servicio cancelado.'
        : 'Confirmá que querés modificar o reabrir este servicio completado.'
      : null
  );

  // Single debounced trigger: attachments are reactive and would fire a second
  // request for every field change, which caused the modal to flicker.
  let firstSuggestionRun = true;
  let lastSuggestionKey = '';

  $effect(() => {
    const params = {
      placeId,
      serviceType,
      date: date.trim(),
      startTime: startTime.trim() || null,
      endTime: endTime.trim() || null,
      excludeServiceId: initial?.id ?? null
    };

    if (placeId <= 0 || !params.date) {
      return;
    }

    const key = JSON.stringify(params);

    if (firstSuggestionRun) {
      firstSuggestionRun = false;
      lastSuggestionKey = key;
      untrack(() => onrequestsuggestions(params));
      return;
    }

    const timer = setTimeout(() => {
      if (key === lastSuggestionKey) {
        return;
      }
      lastSuggestionKey = key;
      onrequestsuggestions(params);
    }, 150);

    return () => clearTimeout(timer);
  });

  function handlePlaceChange(event: Event): void {
    if (initial) {
      return;
    }
    const nextPlaceId = Number((event.currentTarget as HTMLSelectElement).value);
    const nextPlace = places.find((place) => place.id === nextPlaceId);
    if (nextPlace?.recommended_service_type) {
      serviceType = nextPlace.recommended_service_type;
    }
  }

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    submitted = true;
    if (placeError || staffError || timeError || reopenError) {
      return;
    }

    await onsubmit({
      input: {
        place_id: placeId,
        date: date.trim(),
        start_time: startTime.trim() || null,
        end_time: endTime.trim() || null,
        service_type: serviceType,
        status,
        notes: notes.trim() || null,
        staff: selectedStaff
      },
      confirmReopen
    });
  }
</script>

<form id={formId} class="service-form" onsubmit={handleSubmit} aria-busy={busy}>
  <div class="grid-two">
    <FormField id="service-place" label="Lugar" required error={placeError}>
      <select
        id="service-place"
        class="input"
        bind:value={placeId}
        onchange={handlePlaceChange}
        disabled={busy}
      >
        {#each activePlaces as place (place.id)}
          <option value={place.id}>
            {place.name}
            {#if place.recommended_service_type}
              — Sugerido: {labelOf(serviceTypeLabels, place.recommended_service_type)}
            {/if}
          </option>
        {/each}
      </select>
    </FormField>

    <FormField id="service-type" label="Tipo de servicio" required>
      <select
        id="service-type"
        class="input"
        bind:value={serviceType}
        disabled={busy}
      >
        {#each serviceTypeOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </FormField>

    <FormField id="service-date" label="Fecha" required>
      <input
        id="service-date"
        class="input"
        type="date"
        bind:value={date}
        disabled={busy}
      />
    </FormField>

    <FormField id="service-status" label="Estado" required>
      <select id="service-status" class="input" bind:value={status} disabled={busy}>
        {#each serviceStatusOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </FormField>

    <FormField
      id="service-start-time"
      label="Hora de inicio"
      hint="Obligatoria al iniciar o completar."
      error={timeError}
    >
      <input
        id="service-start-time"
        class="input"
        type="time"
        bind:value={startTime}
        disabled={busy}
        aria-invalid={Boolean(timeError)}
      />
    </FormField>

    <FormField
      id="service-end-time"
      label="Hora de finalización"
      hint="Obligatoria al completar."
    >
      <input
        id="service-end-time"
        class="input"
        type="time"
        bind:value={endTime}
        disabled={busy}
        aria-invalid={Boolean(timeError)}
      />
    </FormField>
  </div>

  <AvailabilityList
    {suggestions}
    {allStaff}
    selected={selectedStaff}
    loading={loadingSuggestions}
    disabled={busy}
    onchange={(next) => (selectedStaff = next)}
  />

  {#if staffError}
    <p class="inline-error" role="alert">{staffError}</p>
  {/if}

  <FormField id="service-notes" label="Notas del servicio">
    <textarea
      id="service-notes"
      class="input textarea"
      rows="2"
      bind:value={notes}
      disabled={busy}
    ></textarea>
  </FormField>

  {#if showReopenBox}
    <div class="reopen-box">
      <label class="checkbox-row">
        <input type="checkbox" bind:checked={confirmReopen} disabled={busy} />
        <span>
          {#if isCancelledService}
            Confirmo que quiero reabrir este servicio que figura como cancelado.
          {:else}
            Confirmo que quiero modificar o reabrir este servicio que ya figura como completado.
          {/if}
        </span>
      </label>
      {#if reopenError}
        <p class="inline-error" role="alert">{reopenError}</p>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="form-alert" role="alert">{error}</div>
  {/if}
</form>

<style>
  .service-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .grid-two {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: var(--space-4);
  }

  .textarea {
    min-height: 4.5rem;
    resize: vertical;
  }

  .reopen-box {
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-md);
    background: var(--color-warning-soft);
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: 600;
    cursor: pointer;
  }

  .inline-error {
    color: var(--color-danger);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .form-alert {
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    background: var(--color-danger-soft);
    color: var(--color-ink);
    font-size: var(--text-sm);
  }
</style>
