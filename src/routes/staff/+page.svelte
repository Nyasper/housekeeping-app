<script lang="ts">
  import { onMount } from 'svelte';

  import {
    absenceTypeLabels,
    labelOf,
    staffRoleLabels,
    toOptions,
    weekdayLabels
  } from '#lib/api/labels.js';
  import type {
    Absence,
    AbsenceType,
    Staff,
    StaffInput,
    StaffRole
  } from '#lib/api/types.js';
  import Avatar from '#lib/components/Avatar.svelte';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import EmptyState from '#lib/components/EmptyState.svelte';
  import FormField from '#lib/components/FormField.svelte';
  import StaffForm from '#lib/components/StaffForm.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { staffService } from '#lib/services/staffService.svelte.js';
  import { formatDate, todayIso } from '#lib/utils/format.js';

  const roleOptions = toOptions(staffRoleLabels);
  const absenceOptions = toOptions(absenceTypeLabels);

  let searchInput = $state('');
  let debouncedSearch = $state('');
  let roleFilter = $state<StaffRole | 'all'>('all');
  let statusFilter = $state<'active' | 'inactive' | 'all'>('active');
  let seasonalFilter = $state<'all' | 'seasonal' | 'permanent'>('all');

  let showForm = $state(false);
  let editingStaff = $state<Staff | null>(null);
  let deactivatingStaff = $state<Staff | null>(null);

  let managingAbsencesFor = $state<Staff | null>(null);
  let editingAbsence = $state<Absence | null>(null);
  let absenceType = $state<AbsenceType>('medical_certificate');
  let absenceStart = $state(todayIso());
  let absenceEnd = $state(todayIso());
  let absenceReason = $state('');
  let absenceSubmitted = $state(false);

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

  onMount(() => {
    void staffService.load();
  });

  const filteredStaff = $derived.by(() => {
    return staffService.items.filter((member) => {
      if (statusFilter === 'active' && !member.is_active) {
        return false;
      }
      if (statusFilter === 'inactive' && member.is_active) {
        return false;
      }
      if (roleFilter !== 'all' && member.role !== roleFilter) {
        return false;
      }
      if (seasonalFilter === 'seasonal' && !member.is_seasonal) {
        return false;
      }
      if (seasonalFilter === 'permanent' && member.is_seasonal) {
        return false;
      }
      if (debouncedSearch) {
        const fullName = `${member.first_name} ${member.last_name}`.toLocaleLowerCase('es-UY');
        const nationalId = member.national_id.toLocaleLowerCase('es-UY');
        if (!fullName.includes(debouncedSearch) && !nationalId.includes(debouncedSearch)) {
          return false;
        }
      }
      return true;
    });
  });

  const activeCount = $derived(staffService.items.filter((m) => m.is_active).length);
  const availableTodayCount = $derived(staffService.availableOn(todayIso()).length);

  const absenceDateError = $derived.by(() => {
    if (!absenceSubmitted) {
      return null;
    }
    if (!absenceStart || !absenceEnd) {
      return 'Completá las fechas de inicio y fin de la ausencia.';
    }
    if (absenceStart > absenceEnd) {
      return 'La fecha de inicio no puede ser posterior al fin de la ausencia.';
    }
    return null;
  });

  function openCreateForm(): void {
    editingStaff = null;
    showForm = true;
    staffService.error = null;
  }

  function openEditForm(member: Staff): void {
    editingStaff = member;
    showForm = true;
    staffService.error = null;
  }

  function closeForm(): void {
    showForm = false;
    editingStaff = null;
    staffService.error = null;
  }

  async function handleFormSubmit(payload: {
    input: StaffInput;
    restDays: number[];
    mainPlaceIds: number[];
  }): Promise<void> {
    if (editingStaff) {
      const updated = await staffService.update(
        editingStaff.id,
        payload.input,
        payload.restDays,
        payload.mainPlaceIds
      );
      if (updated) {
        toastTone = 'success';
        toastMessage = 'Datos del funcionario actualizados.';
        closeForm();
      }
    } else {
      const created = await staffService.create(
        payload.input,
        payload.restDays,
        payload.mainPlaceIds
      );
      if (created) {
        toastTone = 'success';
        toastMessage = 'Funcionario registrado.';
        closeForm();
      }
    }
  }

  async function confirmDeactivate(): Promise<void> {
    const target = deactivatingStaff;
    if (!target) {
      return;
    }
    const result = await staffService.deactivate(target.id);
    deactivatingStaff = null;
    if (result) {
      if (staffService.lastWarning) {
        toastTone = 'warning';
        toastMessage = staffService.lastWarning;
      } else {
        toastTone = 'success';
        toastMessage = `${target.first_name} ${target.last_name} fue dado/a de baja.`;
      }
    } else if (staffService.error) {
      toastTone = 'error';
      toastMessage = staffService.error;
    }
  }

  async function handleReactivate(member: Staff): Promise<void> {
    const result = await staffService.reactivate(member.id);
    if (result) {
      toastTone = 'success';
      toastMessage = `${member.first_name} ${member.last_name} fue reactivado/a.`;
    } else if (staffService.error) {
      toastTone = 'error';
      toastMessage = staffService.error;
    }
  }

  function openAbsenceManager(member: Staff): void {
    managingAbsencesFor = member;
    resetAbsenceForm();
    staffService.error = null;
  }

  function resetAbsenceForm(): void {
    editingAbsence = null;
    absenceType = 'medical_certificate';
    absenceStart = todayIso();
    absenceEnd = todayIso();
    absenceReason = '';
    absenceSubmitted = false;
  }

  function startEditAbsence(absence: Absence): void {
    editingAbsence = absence;
    absenceType = absence.type;
    absenceStart = absence.start_date;
    absenceEnd = absence.end_date;
    absenceReason = absence.reason ?? '';
    absenceSubmitted = false;
  }

  async function handleAbsenceSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!managingAbsencesFor) {
      return;
    }
    absenceSubmitted = true;
    if (absenceDateError) {
      return;
    }

    const input = {
      staff_id: managingAbsencesFor.id,
      type: absenceType,
      start_date: absenceStart,
      end_date: absenceEnd,
      reason: absenceReason.trim() || null
    };

    if (editingAbsence) {
      const updated = await staffService.updateAbsence(editingAbsence.id, input);
      if (updated) {
        toastTone = 'success';
        toastMessage = 'Ausencia actualizada.';
        resetAbsenceForm();
      }
    } else {
      const created = await staffService.createAbsence(input);
      if (created) {
        toastTone = 'success';
        toastMessage = 'Ausencia registrada.';
        resetAbsenceForm();
      }
    }
  }

  async function handleDeleteAbsence(absenceId: number): Promise<void> {
    const ok = await staffService.deleteAbsence(absenceId);
    if (ok) {
      toastTone = 'success';
      toastMessage = 'Ausencia eliminada.';
      if (editingAbsence?.id === absenceId) {
        resetAbsenceForm();
      }
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || deactivatingStaff !== null) {
      return;
    }
    if (managingAbsencesFor) {
      managingAbsencesFor = null;
      staffService.error = null;
      return;
    }
    if (showForm) {
      closeForm();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<svelte:head>
  <title>Personal · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">
      {activeCount} activos · {availableTodayCount} disponibles hoy
    </p>
    <h1 class="page-title">Personal</h1>
    <p class="page-lede">
      Administrá mucamas, vidrieros y personal de áreas comunes, sus temporadas, descansos,
      ausencias y apartamentos principales.
    </p>
  </div>
  {#if !showForm}
    <button type="button" class="button button-primary" onclick={openCreateForm}>
      Nuevo funcionario
    </button>
  {/if}
</header>

<Toast message={toastMessage} tone={toastTone} ondismiss={() => (toastMessage = null)} />

{#if showForm}
  <StaffForm
    initial={editingStaff}
    availableApartments={staffService.availableApartments}
    busy={staffService.saving}
    error={staffService.error}
    onsubmit={handleFormSubmit}
    oncancel={closeForm}
  />
{/if}

{#if managingAbsencesFor}
  {@const currentAbsences = staffService.absencesFor(managingAbsencesFor.id)}
  <section class="absence-panel panel" aria-label="Gestión de ausencias">
    <div class="absence-header">
      <div>
        <h2>Ausencias de {managingAbsencesFor.first_name} {managingAbsencesFor.last_name}</h2>
        <p>Registrá certificaciones médicas, licencias o faltas puntuales sin superposiciones.</p>
      </div>
      <button
        type="button"
        class="button button-secondary"
        onclick={() => (managingAbsencesFor = null)}
      >
        Cerrar panel
      </button>
    </div>

    <form class="absence-form" onsubmit={handleAbsenceSubmit}>
      <div class="absence-fields">
        <FormField id="absence-type" label="Tipo de ausencia" required>
          <select id="absence-type" class="input" bind:value={absenceType}>
            {#each absenceOptions as opt (opt.value)}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </FormField>

        <FormField id="absence-start" label="Desde" required error={absenceDateError}>
          <input id="absence-start" class="input" type="date" bind:value={absenceStart} />
        </FormField>

        <FormField id="absence-end" label="Hasta" required>
          <input id="absence-end" class="input" type="date" bind:value={absenceEnd} />
        </FormField>

        <FormField id="absence-reason" label="Motivo u observación">
          <input
            id="absence-reason"
            class="input"
            type="text"
            placeholder="Opcional"
            bind:value={absenceReason}
          />
        </FormField>
      </div>

      {#if staffService.error}
        <p class="inline-error" role="alert">{staffService.error}</p>
      {/if}

      <div class="absence-form-actions">
        <button type="submit" class="button button-primary" disabled={staffService.saving}>
          {editingAbsence ? 'Guardar ausencia' : 'Agregar ausencia'}
        </button>
        {#if editingAbsence}
          <button type="button" class="button button-secondary" onclick={resetAbsenceForm}>
            Cancelar edición
          </button>
        {/if}
      </div>
    </form>

    {#if currentAbsences.length === 0}
      <p class="empty-absences">No hay ausencias registradas para este funcionario.</p>
    {:else}
      <ul class="absence-list">
        {#each currentAbsences as absence (absence.id)}
          <li class="absence-item">
            <div>
              <strong>{labelOf(absenceTypeLabels, absence.type)}</strong>
              <span class="absence-dates">
                {formatDate(absence.start_date)} al {formatDate(absence.end_date)}
              </span>
              {#if absence.reason}
                <p class="absence-reason">{absence.reason}</p>
              {/if}
            </div>
            <div class="absence-item-actions">
              <button
                type="button"
                class="button button-secondary"
                onclick={() => startEditAbsence(absence)}
              >
                Editar
              </button>
              <button
                type="button"
                class="button button-secondary"
                onclick={() => handleDeleteAbsence(absence.id)}
              >
                Eliminar
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

<section class="filters panel" aria-label="Filtros de personal">
  <div class="filter-field filter-search">
    <label for="staff-search">Buscar por nombre o cédula</label>
    <input
      id="staff-search"
      class="input"
      type="search"
      placeholder="Ej. María Gómez o 41234567"
      value={searchInput}
      oninput={handleSearchInput}
    />
  </div>

  <div class="filter-field">
    <label for="staff-filter-role">Rol</label>
    <select id="staff-filter-role" class="input" bind:value={roleFilter}>
      <option value="all">Todos los roles</option>
      {#each roleOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </div>

  <div class="filter-field">
    <label for="staff-filter-status">Estado</label>
    <select id="staff-filter-status" class="input" bind:value={statusFilter}>
      <option value="active">Activos</option>
      <option value="inactive">Dados de baja</option>
      <option value="all">Todos</option>
    </select>
  </div>

  <div class="filter-field">
    <label for="staff-filter-seasonal">Modalidad</label>
    <select id="staff-filter-seasonal" class="input" bind:value={seasonalFilter}>
      <option value="all">Todas</option>
      <option value="permanent">Permanente</option>
      <option value="seasonal">Zafral</option>
    </select>
  </div>
</section>

{#if staffService.error && !showForm && !managingAbsencesFor}
  <section class="panel error-banner" role="alert">
    <p>{staffService.error}</p>
    <button type="button" class="button button-secondary" onclick={() => staffService.load()}>
      Reintentar
    </button>
  </section>
{/if}

{#if staffService.loading && staffService.items.length === 0}
  <section class="panel status-panel" role="status">
    <p>Cargando personal…</p>
  </section>
{:else if filteredStaff.length === 0}
  <section class="panel status-panel">
    <EmptyState
      title="No se encontraron funcionarios"
      description={staffService.items.length === 0
        ? 'Registrá al primer integrante del equipo de limpieza para asignarle apartamentos y servicios.'
        : 'Probá cambiar los filtros de búsqueda o el estado seleccionado.'}
    >
      {#if staffService.items.length === 0 && !showForm}
        <button type="button" class="button button-primary" onclick={openCreateForm}>
          Registrar primer funcionario
        </button>
      {/if}
    </EmptyState>
  </section>
{:else}
  <div class="staff-grid">
    {#each filteredStaff as member (member.id)}
      {@const memberAbsences = staffService.absencesFor(member.id)}
      {@const availableToday = staffService.isAvailableOn(member, todayIso())}
      <article class={`staff-card panel ${!member.is_active ? 'is-inactive' : ''}`}>
        <div class="card-top">
          <Avatar
            photoUrl={member.photo_url}
            gender={member.gender}
            alt={`Foto de ${member.first_name} ${member.last_name}`}
            size="md"
          />
          <div class="card-identity">
            <div class="name-row">
              <h2>{member.first_name} {member.last_name}</h2>
              <span class="badge badge-role">{labelOf(staffRoleLabels, member.role)}</span>
              {#if !member.is_active}
                <span class="badge badge-inactive">Dado de baja</span>
              {:else if availableToday}
                <span class="badge badge-available">Disponible hoy</span>
              {:else}
                <span class="badge badge-unavailable">No disponible hoy</span>
              {/if}
              {#if member.is_seasonal}
                <span class="badge badge-seasonal">Zafral</span>
              {/if}
            </div>
            <p class="meta-line">
              C.I. {member.national_id}
              {#if member.phone}
                · Tel. {member.phone}
              {/if}
            </p>
          </div>
        </div>

        <dl class="card-details">
          {#if member.is_seasonal}
            <div>
              <dt>Temporada</dt>
              <dd>
                {#if member.season_start_date && member.season_end_date}
                  {formatDate(member.season_start_date)} al {formatDate(member.season_end_date)}
                {:else}
                  Sin fechas definidas
                {/if}
              </dd>
            </div>
          {/if}

          <div>
            <dt>Descanso semanal</dt>
            <dd>
              {#if member.rest_days.length === 0}
                Sin días fijos
              {:else}
                {member.rest_days.map((d) => weekdayLabels[d] ?? '').join(', ')}
              {/if}
            </dd>
          </div>

          <div>
            <dt>Apartamentos principales</dt>
            <dd>
              {#if member.main_places.length === 0}
                Ninguno asignado
              {:else}
                {member.main_places.map((p) => p.name).join(', ')}
              {/if}
            </dd>
          </div>
        </dl>

        {#if member.notes}
          <p class="card-notes">{member.notes}</p>
        {/if}

        <div class="card-actions">
          <button
            type="button"
            class="button button-secondary"
            onclick={() => openEditForm(member)}
          >
            Editar
          </button>
          <button
            type="button"
            class="button button-secondary"
            onclick={() => openAbsenceManager(member)}
          >
            Ausencias ({memberAbsences.length})
          </button>
          {#if member.is_active}
            <button
              type="button"
              class="button button-secondary"
              onclick={() => (deactivatingStaff = member)}
            >
              Dar de baja
            </button>
          {:else}
            <button
              type="button"
              class="button button-primary"
              onclick={() => handleReactivate(member)}
            >
              Reactivar
            </button>
          {/if}
        </div>
      </article>
    {/each}
  </div>
{/if}

<ConfirmDialog
  open={deactivatingStaff !== null}
  title="Dar de baja al funcionario"
  description={deactivatingStaff
    ? `¿Querés dar de baja a ${deactivatingStaff.first_name} ${deactivatingStaff.last_name}? Su historial de servicios se conservará y podrás reactivarlo/a cuando lo necesites.`
    : ''}
  confirmLabel="Dar de baja"
  tone="danger"
  busy={staffService.saving}
  onconfirm={confirmDeactivate}
  oncancel={() => (deactivatingStaff = null)}
/>

<style>
  .filters {
    display: grid;
    grid-template-columns: 2fr repeat(3, minmax(10rem, 1fr));
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    margin-bottom: var(--space-5);
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

  .status-panel {
    padding: var(--space-6);
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

  .staff-grid {
    display: grid;
    gap: var(--space-4);
  }

  .staff-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .staff-card.is-inactive {
    opacity: 0.78;
  }

  .card-top {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .card-identity {
    flex: 1;
    min-width: 0;
  }

  .name-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .name-row h2 {
    font-size: var(--text-lg);
  }

  .meta-line {
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

  .badge-role {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-available {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .badge-unavailable {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .badge-inactive {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-seasonal {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .card-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: var(--space-3);
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
    border: 1px solid var(--color-line);
  }

  .card-details dt {
    font-size: var(--text-xs);
    font-weight: 650;
    color: var(--color-ink-muted);
  }

  .card-details dd {
    margin: 0.2rem 0 0;
    font-size: var(--text-sm);
    font-weight: 550;
  }

  .card-notes {
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

  .absence-panel {
    padding: var(--space-5);
    margin-bottom: var(--space-5);
  }

  .absence-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-line);
  }

  .absence-header h2 {
    font-size: var(--text-xl);
  }

  .absence-header p {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .absence-form {
    padding: var(--space-4) 0;
    border-bottom: 1px solid var(--color-line);
  }

  .absence-fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: var(--space-3);
  }

  .absence-form-actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-4);
  }

  .inline-error {
    margin-top: var(--space-3);
    color: var(--color-danger);
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .empty-absences {
    padding-top: var(--space-4);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .absence-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: var(--space-4) 0 0;
    padding: 0;
    list-style: none;
  }

  .absence-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
  }

  .absence-dates {
    margin-left: var(--space-2);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .absence-reason {
    margin-top: var(--space-1);
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
  }

  .absence-item-actions {
    display: flex;
    gap: var(--space-2);
  }

  @media (max-width: 900px) {
    .filters {
      grid-template-columns: 1fr;
    }
  }
</style>
