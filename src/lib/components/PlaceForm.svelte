<script lang="ts">
  import {
    placeStatusLabels,
    placeTypeLabels,
    toOptions
  } from '#lib/api/labels.js';
  import type {
    Place,
    PlaceInput,
    PlaceStatus,
    PlaceType
  } from '#lib/api/types.js';
  import FormField from './FormField.svelte';

  interface Props {
    initial?: Place | null;
    busy?: boolean;
    error?: string | null;
    onsubmit: (input: PlaceInput) => Promise<void> | void;
    oncancel: () => void;
  }

  let { initial = null, busy = false, error = null, onsubmit, oncancel }: Props = $props();

  const typeOptions = toOptions(placeTypeLabels);
  const statusOptions = toOptions(placeStatusLabels);

  let name = $derived(initial?.name ?? '');
  let placeType = $derived<PlaceType>(initial?.type ?? 'apartment');
  let statusOverride = $derived<PlaceStatus | 'auto'>(initial?.status ?? 'auto');
  let location = $derived(initial?.location ?? '');
  let checkInDate = $derived(initial?.check_in_date ?? '');
  let checkOutDate = $derived(initial?.check_out_date ?? '');
  let notes = $derived(initial?.notes ?? '');
  let submitted = $derived(Boolean(initial && false));

  const nameError = $derived(
    submitted && !name.trim() ? 'Ingresá el nombre del apartamento o área común.' : null
  );

  const dateError = $derived.by(() => {
    if (!submitted || placeType !== 'apartment') {
      return null;
    }
    const start = checkInDate.trim();
    const end = checkOutDate.trim();
    if (start && end && start > end) {
      return 'La fecha de ingreso no puede ser posterior a la fecha de egreso.';
    }
    return null;
  });

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    submitted = true;
    if (nameError || dateError) {
      return;
    }

    const isApartment = placeType === 'apartment';
    await onsubmit({
      name: name.trim(),
      type: placeType,
      status: isApartment && statusOverride !== 'auto' ? statusOverride : null,
      location: location.trim() || null,
      check_in_date: isApartment ? checkInDate.trim() || null : null,
      check_out_date: isApartment ? checkOutDate.trim() || null : null,
      notes: notes.trim() || null
    });
  }
</script>

<form class="place-form panel" onsubmit={handleSubmit} aria-busy={busy}>
  <div class="form-header">
    <div>
      <h2>{initial ? 'Editar lugar' : 'Nuevo lugar'}</h2>
      <p>
        Configurá los datos del apartamento o área común, su ubicación y las fechas de estadía si se
        conocen.
      </p>
    </div>
    <button type="button" class="button button-secondary" onclick={oncancel} disabled={busy}>
      Cerrar
    </button>
  </div>

  <div class="grid-two">
    <FormField
      id="place-name"
      label="Nombre"
      required
      hint="Único en el edificio (ej. Apto 101, Lobby, Piscina)."
      error={nameError}
    >
      <input
        id="place-name"
        class="input"
        type="text"
        bind:value={name}
        disabled={busy}
        aria-invalid={Boolean(nameError)}
      />
    </FormField>

    <FormField id="place-type" label="Tipo de lugar" required>
      <select id="place-type" class="input" bind:value={placeType} disabled={busy}>
        {#each typeOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </FormField>

    <FormField id="place-location" label="Piso o ubicación">
      <input
        id="place-location"
        class="input"
        type="text"
        placeholder="Ej. Piso 3, Torre Norte"
        bind:value={location}
        disabled={busy}
      />
    </FormField>

    {#if placeType === 'apartment'}
      <FormField
        id="place-status"
        label="Estado del apartamento"
        hint="En modo automático se calcula según las fechas de ingreso y egreso."
      >
        <select id="place-status" class="input" bind:value={statusOverride} disabled={busy}>
          <option value="auto">Automático (según fechas)</option>
          {#each statusOptions as option (option.value)}
            <option value={option.value}>{option.label} (manual)</option>
          {/each}
        </select>
      </FormField>
    {/if}
  </div>

  {#if placeType === 'apartment'}
    <fieldset class="fieldset">
      <legend>Estadía actual o próxima (opcional)</legend>
      <p class="fieldset-note">
        Dejá en blanco cualquier fecha que no conozcas; en la pantalla se mostrará como «Sin datos».
      </p>
      <div class="grid-two">
        <FormField id="place-check-in" label="Fecha de ingreso (check-in)" error={dateError}>
          <div class="date-input-row">
            <input
              id="place-check-in"
              class="input"
              type="date"
              bind:value={checkInDate}
              disabled={busy}
              aria-invalid={Boolean(dateError)}
            />
            {#if checkInDate}
              <button
                type="button"
                class="button button-secondary"
                onclick={() => (checkInDate = '')}
                disabled={busy}
              >
                Limpiar
              </button>
            {/if}
          </div>
        </FormField>

        <FormField id="place-check-out" label="Fecha de egreso (check-out)">
          <div class="date-input-row">
            <input
              id="place-check-out"
              class="input"
              type="date"
              bind:value={checkOutDate}
              disabled={busy}
              aria-invalid={Boolean(dateError)}
            />
            {#if checkOutDate}
              <button
                type="button"
                class="button button-secondary"
                onclick={() => (checkOutDate = '')}
                disabled={busy}
              >
                Limpiar
              </button>
            {/if}
          </div>
        </FormField>
      </div>
    </fieldset>
  {/if}

  <FormField id="place-notes" label="Notas">
    <textarea
      id="place-notes"
      class="input textarea"
      rows="3"
      bind:value={notes}
      disabled={busy}
    ></textarea>
  </FormField>

  {#if error}
    <div class="form-alert" role="alert">{error}</div>
  {/if}

  <div class="form-actions">
    <button type="submit" class="button button-primary" disabled={busy}>
      {busy ? 'Guardando…' : initial ? 'Guardar cambios' : 'Crear lugar'}
    </button>
    <button type="button" class="button button-secondary" onclick={oncancel} disabled={busy}>
      Cancelar
    </button>
  </div>
</form>

<style>
  .place-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-6);
    margin-bottom: var(--space-6);
  }

  .form-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-line);
  }

  .form-header h2 {
    font-size: var(--text-xl);
  }

  .form-header p {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .grid-two {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(15rem, 1fr));
    gap: var(--space-4);
  }

  .fieldset {
    margin: 0;
    padding: var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-lg);
  }

  .fieldset legend {
    padding: 0 var(--space-2);
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .fieldset-note {
    margin-bottom: var(--space-4);
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .date-input-row {
    display: flex;
    gap: var(--space-2);
  }

  .textarea {
    min-height: 5rem;
    resize: vertical;
  }

  .form-alert {
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    background: var(--color-danger-soft);
    color: var(--color-ink);
    font-size: var(--text-sm);
  }

  .form-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    padding-top: var(--space-4);
    border-top: 1px solid var(--color-line);
  }
</style>
