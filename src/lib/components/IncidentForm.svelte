<script lang="ts">
  import { open as openFileDialog } from '@tauri-apps/plugin-dialog';
  import { untrack } from 'svelte';

  import { isTauriRuntime } from '#lib/api/invoke.js';
  import {
    incidentTypeLabels,
    labelOf,
    serviceStatusLabels,
    serviceTypeLabels,
    toOptions
  } from '#lib/api/labels.js';
  import type {
    CleaningService,
    IncidentInput,
    IncidentType
  } from '#lib/api/types.js';
  import FormField from './FormField.svelte';

  interface Props {
    formId: string;
    services: CleaningService[];
    initialServiceId?: number | null;
    defaultDate: string;
    busy?: boolean;
    error?: string | null;
    dirty?: boolean;
    onsubmit: (input: IncidentInput) => Promise<void> | void;
  }

  let {
    formId,
    services,
    initialServiceId = null,
    defaultDate,
    busy = false,
    error = null,
    dirty = $bindable(false),
    onsubmit
  }: Props = $props();

  const incidentTypeOptions = toOptions(incidentTypeLabels);

  const eligibleServices = $derived(
    services.filter((service) => service.status !== 'cancelled')
  );

  let serviceId = $derived<number>(
    initialServiceId && eligibleServices.some((s) => s.id === initialServiceId)
      ? initialServiceId
      : (eligibleServices[0]?.id ?? 0)
  );
  let incidentType = $state<IncidentType>('damage');
  let date = $derived(
    eligibleServices.find((s) => s.id === serviceId)?.date ?? defaultDate
  );
  let description = $state('');
  let photoSourcePath = $state<string | null>(null);
  let submitted = $state(false);

  const baseline = untrack(() => ({
    serviceId,
    incidentType,
    date,
    description,
    photoSourcePath
  }));

  const isDirty = $derived(
    serviceId !== baseline.serviceId ||
      incidentType !== baseline.incidentType ||
      date !== baseline.date ||
      description !== baseline.description ||
      photoSourcePath !== baseline.photoSourcePath
  );

  $effect(() => {
    dirty = isDirty;
  });

  const serviceError = $derived(
    submitted && !serviceId
      ? 'Seleccioná un servicio de limpieza activo o completado.'
      : null
  );

  const descriptionError = $derived(
    submitted && !description.trim()
      ? 'Describí la incidencia encontrada durante o después del servicio.'
      : null
  );

  const dateError = $derived(
    submitted && !date.trim() ? 'Indicá la fecha de la incidencia.' : null
  );

  const selectedFileName = $derived.by(() => {
    if (!photoSourcePath) {
      return null;
    }
    const parts = photoSourcePath.split(/[/\\]/);
    return parts[parts.length - 1] ?? photoSourcePath;
  });

  function handleServiceChange(): void {
    const selected = eligibleServices.find((s) => s.id === serviceId);
    if (selected) {
      date = selected.date;
    }
  }

  async function pickPhoto(): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    const selected = await openFileDialog({
      multiple: false,
      title: 'Seleccionar foto de la incidencia',
      filters: [
        {
          name: 'Imágenes',
          extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif']
        }
      ]
    });
    if (typeof selected === 'string' && selected.trim()) {
      photoSourcePath = selected;
    }
  }

  function clearPhoto(): void {
    photoSourcePath = null;
  }

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    submitted = true;
    if (serviceError || descriptionError || dateError) {
      return;
    }

    await onsubmit({
      service_id: serviceId,
      type: incidentType,
      description: description.trim(),
      date: date.trim(),
      photo_source_path: photoSourcePath
    });
  }
</script>

<form id={formId} class="incident-form" onsubmit={handleSubmit} aria-busy={busy}>
  {#if eligibleServices.length === 0}
    <p class="empty-services" role="status">
      No hay servicios no cancelados en esta fecha sobre los cuales registrar una incidencia. Creá
      primero un servicio de limpieza.
    </p>
  {:else}
    <div class="grid-three">
      <FormField id="incident-service" label="Servicio asociado" required error={serviceError}>
        <select
          id="incident-service"
          class="input"
          bind:value={serviceId}
          onchange={handleServiceChange}
          disabled={busy}
        >
          {#each eligibleServices as service (service.id)}
            <option value={service.id}>
              {service.place_name} · {labelOf(serviceTypeLabels, service.service_type)} ({labelOf(
                serviceStatusLabels,
                service.status
              )})
            </option>
          {/each}
        </select>
      </FormField>

      <FormField id="incident-type" label="Tipo de incidencia" required>
        <select id="incident-type" class="input" bind:value={incidentType} disabled={busy}>
          {#each incidentTypeOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </FormField>

      <FormField id="incident-date" label="Fecha" required error={dateError}>
        <input
          id="incident-date"
          class="input"
          type="date"
          bind:value={date}
          disabled={busy}
          aria-invalid={Boolean(dateError)}
        />
      </FormField>
    </div>

    <FormField
      id="incident-description"
      label="Descripción"
      required
      hint="Detallá qué ocurrió o qué hace falta para resolverlo."
      error={descriptionError}
    >
      <textarea
        id="incident-description"
        class="input textarea"
        rows="3"
        placeholder="Ej. Control remoto del aire acondicionado sin tapa, o billetera olvidada en la mesa de luz."
        bind:value={description}
        disabled={busy}
        aria-invalid={Boolean(descriptionError)}
      ></textarea>
    </FormField>

    <div class="photo-row">
      <div class="photo-info">
        <span class="photo-label">Foto opcional</span>
        <span class="photo-sub">
          {selectedFileName
            ? `Archivo seleccionado: ${selectedFileName}`
            : 'Adjuntá una imagen si ayuda a identificar el daño u objeto.'}
        </span>
      </div>
      <div class="photo-actions">
        <button type="button" class="button button-secondary" onclick={pickPhoto} disabled={busy}>
          Elegir imagen…
        </button>
        {#if photoSourcePath}
          <button
            type="button"
            class="button button-secondary"
            onclick={clearPhoto}
            disabled={busy}
          >
            Quitar imagen
          </button>
        {/if}
      </div>
    </div>
  {/if}

  {#if error}
    <div class="form-alert" role="alert">{error}</div>
  {/if}
</form>

<style>
  .incident-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .empty-services {
    padding: var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
  }

  .grid-three {
    display: grid;
    grid-template-columns: 1.5fr 1fr 1fr;
    gap: var(--space-4);
  }

  .textarea {
    min-height: 5rem;
    resize: vertical;
  }

  .photo-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
  }

  .photo-info {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .photo-label {
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .photo-sub {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .photo-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .form-alert {
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    background: var(--color-danger-soft);
    color: var(--color-ink);
    font-size: var(--text-sm);
  }

  @media (max-width: 900px) {
    .grid-three {
      grid-template-columns: 1fr;
    }
  }
</style>
