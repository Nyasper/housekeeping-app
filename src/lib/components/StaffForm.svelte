<script lang="ts">
  import { open as openFileDialog } from '@tauri-apps/plugin-dialog';

  import { isTauriRuntime } from '#lib/api/invoke.js';
  import {
    genderLabels,
    staffRoleLabels,
    toOptions,
    weekdayLabels
  } from '#lib/api/labels.js';
  import type {
    Gender,
    PlaceSummary,
    Staff,
    StaffInput,
    StaffRole
  } from '#lib/api/types.js';
  import Avatar from './Avatar.svelte';
  import FormField from './FormField.svelte';

  interface Props {
    initial?: Staff | null;
    availableApartments: PlaceSummary[];
    busy?: boolean;
    error?: string | null;
    onsubmit: (payload: {
      input: StaffInput;
      restDays: number[];
      mainPlaceIds: number[];
    }) => Promise<void> | void;
    oncancel: () => void;
  }

  let {
    initial = null,
    availableApartments,
    busy = false,
    error = null,
    onsubmit,
    oncancel
  }: Props = $props();

  const genderOptions = toOptions(genderLabels);
  const roleOptions = toOptions(staffRoleLabels);

  let firstName = $derived(initial?.first_name ?? '');
  let lastName = $derived(initial?.last_name ?? '');
  let nationalId = $derived(initial?.national_id ?? '');
  let birthDate = $derived(initial?.birth_date ?? '');
  let phone = $derived(initial?.phone ?? '');
  let address = $derived(initial?.address ?? '');
  let gender = $derived<Gender>(initial?.gender ?? 'female');
  let role = $derived<StaffRole>(initial?.role ?? 'maid');
  let isSeasonal = $derived(initial?.is_seasonal ?? false);
  let seasonStartDate = $derived(initial?.season_start_date ?? '');
  let seasonEndDate = $derived(initial?.season_end_date ?? '');
  let notes = $derived(initial?.notes ?? '');
  let restDays = $derived<number[]>(initial ? [...initial.rest_days] : []);
  let mainPlaceIds = $derived<number[]>(initial ? initial.main_places.map((p) => p.id) : []);
  let photoSourcePath = $derived<string | null>(initial ? null : null);
  let removePhoto = $derived(Boolean(initial && false));
  let submitted = $derived(Boolean(initial && false));

  const firstNameError = $derived(
    submitted && !firstName.trim() ? 'Ingresá el nombre del funcionario.' : null
  );
  const lastNameError = $derived(
    submitted && !lastName.trim() ? 'Ingresá el apellido del funcionario.' : null
  );
  const nationalIdError = $derived(
    submitted && !nationalId.trim() ? 'Ingresá la cédula de identidad.' : null
  );

  const seasonError = $derived.by(() => {
    if (!submitted || !isSeasonal) {
      return null;
    }
    const start = seasonStartDate.trim();
    const end = seasonEndDate.trim();
    if (!start && !end) {
      return null;
    }
    if (!start || !end) {
      return 'Completá tanto el inicio como el fin de la temporada, o dejá ambos vacíos.';
    }
    if (start > end) {
      return 'La fecha de inicio no puede ser posterior al fin de la temporada.';
    }
    return null;
  });

  const previewPhotoUrl = $derived.by(() => {
    if (removePhoto) {
      return null;
    }
    if (photoSourcePath) {
      return photoSourcePath;
    }
    return initial?.photo_url ?? null;
  });

  function toggleRestDay(day: number): void {
    if (restDays.includes(day)) {
      restDays = restDays.filter((item) => item !== day);
    } else {
      restDays = [...restDays, day].sort((a, b) => a - b);
    }
  }

  function toggleMainPlace(placeId: number): void {
    if (mainPlaceIds.includes(placeId)) {
      mainPlaceIds = mainPlaceIds.filter((id) => id !== placeId);
    } else {
      mainPlaceIds = [...mainPlaceIds, placeId];
    }
  }

  async function pickPhoto(): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    const selected = await openFileDialog({
      multiple: false,
      title: 'Seleccionar foto del funcionario',
      filters: [
        {
          name: 'Imágenes',
          extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif']
        }
      ]
    });
    if (typeof selected === 'string' && selected.trim()) {
      photoSourcePath = selected;
      removePhoto = false;
    }
  }

  function clearPhoto(): void {
    photoSourcePath = null;
    removePhoto = true;
  }

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    submitted = true;
    if (firstNameError || lastNameError || nationalIdError || seasonError) {
      return;
    }

    await onsubmit({
      input: {
        first_name: firstName.trim(),
        last_name: lastName.trim(),
        national_id: nationalId.trim(),
        birth_date: birthDate.trim() || null,
        phone: phone.trim() || null,
        address: address.trim() || null,
        gender,
        role,
        is_seasonal: isSeasonal,
        season_start_date: isSeasonal ? seasonStartDate.trim() || null : null,
        season_end_date: isSeasonal ? seasonEndDate.trim() || null : null,
        notes: notes.trim() || null,
        photo_source_path: photoSourcePath,
        remove_photo: removePhoto
      },
      restDays,
      mainPlaceIds
    });
  }
</script>

<form class="staff-form panel" onsubmit={handleSubmit} aria-busy={busy}>
  <div class="form-header">
    <div>
      <h2>{initial ? 'Editar funcionario' : 'Nuevo funcionario'}</h2>
      <p>
        Cargá los datos personales, el rol, los días de descanso y los apartamentos principales.
      </p>
    </div>
    <button type="button" class="button button-secondary" onclick={oncancel} disabled={busy}>
      Cerrar
    </button>
  </div>

  <div class="photo-row">
    <Avatar
      photoUrl={previewPhotoUrl}
      {gender}
      alt={`Foto de ${firstName || 'funcionario'}`}
      size="lg"
    />
    <div class="photo-actions">
      <div class="photo-buttons">
        <button type="button" class="button button-secondary" onclick={pickPhoto} disabled={busy}>
          Elegir foto…
        </button>
        {#if previewPhotoUrl || initial?.photo_path}
          <button
            type="button"
            class="button button-secondary"
            onclick={clearPhoto}
            disabled={busy}
          >
            Usar avatar por defecto
          </button>
        {/if}
      </div>
      <p class="photo-hint">
        Si no elegís una foto, se muestra el avatar predeterminado según el género seleccionado.
      </p>
    </div>
  </div>

  <div class="grid-two">
    <FormField id="staff-first-name" label="Nombre" required error={firstNameError}>
      <input
        id="staff-first-name"
        class="input"
        type="text"
        bind:value={firstName}
        disabled={busy}
        aria-invalid={Boolean(firstNameError)}
      />
    </FormField>

    <FormField id="staff-last-name" label="Apellido" required error={lastNameError}>
      <input
        id="staff-last-name"
        class="input"
        type="text"
        bind:value={lastName}
        disabled={busy}
        aria-invalid={Boolean(lastNameError)}
      />
    </FormField>

    <FormField
      id="staff-national-id"
      label="Cédula de identidad"
      required
      hint="Única por persona, incluso si fue dada de baja."
      error={nationalIdError}
    >
      <input
        id="staff-national-id"
        class="input"
        type="text"
        bind:value={nationalId}
        disabled={busy}
        aria-invalid={Boolean(nationalIdError)}
      />
    </FormField>

    <FormField id="staff-birth-date" label="Fecha de nacimiento">
      <input
        id="staff-birth-date"
        class="input"
        type="date"
        bind:value={birthDate}
        disabled={busy}
      />
    </FormField>

    <FormField id="staff-phone" label="Teléfono">
      <input id="staff-phone" class="input" type="tel" bind:value={phone} disabled={busy} />
    </FormField>

    <FormField id="staff-address" label="Dirección">
      <input id="staff-address" class="input" type="text" bind:value={address} disabled={busy} />
    </FormField>

    <FormField id="staff-gender" label="Género" required>
      <select id="staff-gender" class="input" bind:value={gender} disabled={busy}>
        {#each genderOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </FormField>

    <FormField id="staff-role" label="Rol" required>
      <select id="staff-role" class="input" bind:value={role} disabled={busy}>
        {#each roleOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </FormField>
  </div>

  <fieldset class="fieldset">
    <legend>Modalidad zafral y temporada</legend>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={isSeasonal} disabled={busy} />
      <span>Personal zafral (trabaja por temporada)</span>
    </label>

    {#if isSeasonal}
      <div class="grid-two season-dates">
        <FormField
          id="staff-season-start"
          label="Inicio de temporada"
          hint="Opcional. Si cargás una fecha, completá ambas."
        >
          <input
            id="staff-season-start"
            class="input"
            type="date"
            bind:value={seasonStartDate}
            disabled={busy}
            aria-invalid={Boolean(seasonError)}
          />
        </FormField>

        <FormField id="staff-season-end" label="Fin de temporada">
          <input
            id="staff-season-end"
            class="input"
            type="date"
            bind:value={seasonEndDate}
            disabled={busy}
            aria-invalid={Boolean(seasonError)}
          />
        </FormField>
      </div>
      {#if seasonError}
        <p class="season-error" role="alert">{seasonError}</p>
      {/if}
    {/if}
  </fieldset>

  <fieldset class="fieldset">
    <legend>Días de descanso semanal</legend>
    <div class="pill-grid">
      {#each weekdayLabels as label, index (index)}
        <label class={`pill-choice ${restDays.includes(index) ? 'is-selected' : ''}`}>
          <input
            type="checkbox"
            checked={restDays.includes(index)}
            onchange={() => toggleRestDay(index)}
            disabled={busy}
          />
          <span>{label}</span>
        </label>
      {/each}
    </div>
  </fieldset>

  <fieldset class="fieldset">
    <legend>Apartamentos principales asignados</legend>
    {#if availableApartments.length === 0}
      <p class="empty-note">
        Todavía no hay apartamentos activos cargados. Podés crearlos en la sección Lugares y
        asignarlos luego.
      </p>
    {:else}
      <div class="pill-grid">
        {#each availableApartments as apartment (apartment.id)}
          <label
            class={`pill-choice ${mainPlaceIds.includes(apartment.id) ? 'is-selected' : ''}`}
          >
            <input
              type="checkbox"
              checked={mainPlaceIds.includes(apartment.id)}
              onchange={() => toggleMainPlace(apartment.id)}
              disabled={busy}
            />
            <span>{apartment.name}</span>
          </label>
        {/each}
      </div>
    {/if}
  </fieldset>

  <FormField id="staff-notes" label="Notas">
    <textarea
      id="staff-notes"
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
      {busy ? 'Guardando…' : initial ? 'Guardar cambios' : 'Crear funcionario'}
    </button>
    <button type="button" class="button button-secondary" onclick={oncancel} disabled={busy}>
      Cancelar
    </button>
  </div>
</form>

<style>
  .staff-form {
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

  .photo-row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .photo-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .photo-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .photo-hint {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
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

  .checkbox-row {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: 600;
    cursor: pointer;
  }

  .season-dates {
    margin-top: var(--space-4);
  }

  .season-error {
    margin-top: var(--space-2);
    color: var(--color-danger);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .pill-grid {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .pill-choice {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 2.35rem;
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    font-size: var(--text-sm);
    cursor: pointer;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast);
  }

  .pill-choice.is-selected {
    border-color: var(--color-primary);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-weight: 650;
  }

  .empty-note {
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .textarea {
    min-height: 5.5rem;
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
