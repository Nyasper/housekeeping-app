<script lang="ts">
  import { labelOf, staffRoleLabels } from '#lib/api/labels.js';
  import type {
    ServiceStaffInput,
    Staff,
    StaffSuggestion
  } from '#lib/api/types.js';
  import Avatar from './Avatar.svelte';

  interface Props {
    suggestions: StaffSuggestion[];
    allStaff: Staff[];
    selected: ServiceStaffInput[];
    loading?: boolean;
    disabled?: boolean;
    onchange: (next: ServiceStaffInput[]) => void;
  }

  let {
    suggestions,
    allStaff,
    selected,
    loading = false,
    disabled = false,
    onchange
  }: Props = $props();

  let showAllStaff = $state(false);

  const selectedIds = $derived(new Set(selected.map((item) => item.staff_id)));
  const suggestedIds = $derived(new Set(suggestions.map((item) => item.staff.id)));

  const selectedActiveMembers = $derived(
    selected
      .map((item) => allStaff.find((member) => member.id === item.staff_id))
      .filter(
        (member): member is Staff =>
          member !== undefined && member.is_active && !suggestedIds.has(member.id)
      )
  );

  const extraActiveStaff = $derived(
    allStaff.filter(
      (member) =>
        member.is_active && !suggestedIds.has(member.id) && !selectedIds.has(member.id)
    )
  );

  const selectedInactiveMembers = $derived(
    selected
      .map((item) => allStaff.find((member) => member.id === item.staff_id))
      .filter((member): member is Staff => member !== undefined && !member.is_active)
  );

  function isSelected(staffId: number): boolean {
    return selectedIds.has(staffId);
  }

  function isLead(staffId: number): boolean {
    return selected.some((item) => item.staff_id === staffId && item.is_lead);
  }

  function toggleStaff(staffId: number): void {
    if (disabled) {
      return;
    }
    if (isSelected(staffId)) {
      const next = selected.filter((item) => item.staff_id !== staffId);
      onchange(next);
    } else {
      const shouldBeLead = selected.length === 0;
      onchange([...selected, { staff_id: staffId, is_lead: shouldBeLead }]);
    }
  }

  function setLead(staffId: number): void {
    if (disabled) {
      return;
    }
    const currentlyLead = isLead(staffId);
    const next = selected.map((item) => ({
      staff_id: item.staff_id,
      is_lead: item.staff_id === staffId ? !currentlyLead : false
    }));
    onchange(next);
  }
</script>

<div class="availability-list">
  <div class="section-header">
    <div>
      <h3>Sugerencias de asignación asistida</h3>
      <p>
        Ordenadas por compatibilidad de rol, apartamento principal y menor carga reciente.
      </p>
    </div>
    <span class="selected-counter">
      {#if loading && suggestions.length > 0}
        <span role="status">Actualizando…</span>
      {:else}
        {selected.length} seleccionado{selected.length === 1 ? '' : 's'}
      {/if}
    </span>
  </div>

  {#if selectedInactiveMembers.length > 0}
    <div class="inactive-assigned" role="group" aria-label="Personal dado de baja asignado">
      <p class="status-note">
        Estos funcionarios están dados de baja pero siguen asignados al servicio. Quitalos o
        reasignalos antes de guardar.
      </p>
      <ul class="candidate-grid">
        {#each selectedInactiveMembers as member (member.id)}
          {@const lead = isLead(member.id)}
          <li class="candidate-card is-selected is-inactive">
            <div class="candidate-main">
              <label class="candidate-check">
                <input
                  type="checkbox"
                  checked
                  onchange={() => toggleStaff(member.id)}
                  {disabled}
                />
                <Avatar
                  photoUrl={member.photo_url}
                  gender={member.gender}
                  alt={`Foto de ${member.first_name} ${member.last_name}`}
                  size="sm"
                />
                <span class="candidate-info">
                  <span class="candidate-name">
                    {member.first_name} {member.last_name}
                  </span>
                  <span class="candidate-role">
                    {labelOf(staffRoleLabels, member.role)}
                  </span>
                </span>
              </label>

              {#if lead}
                <button
                  type="button"
                  class="lead-toggle is-lead"
                  onclick={() => setLead(member.id)}
                  {disabled}
                >
                  Responsable principal
                </button>
              {/if}
            </div>

            <div class="reasons-row">
              <span class="reason-pill is-warning">Dado de baja</span>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if loading && suggestions.length === 0}
    <p class="status-note" role="status">Calculando sugerencias disponibles…</p>
  {:else if suggestions.length === 0}
    <p class="status-note">
      No hay personal disponible sugerido automáticamente para esta fecha y horario. Podés elegir
      manualmente entre el personal activo debajo.
    </p>
  {:else}
    <ul class="candidate-grid" aria-busy={loading}>
      {#each suggestions as item, index (item.staff.id)}
        {@const checked = isSelected(item.staff.id)}
        {@const lead = isLead(item.staff.id)}
        <li class={`candidate-card ${checked ? 'is-selected' : ''}`}>
          <div class="candidate-main">
            <label class="candidate-check">
              <input
                type="checkbox"
                {checked}
                onchange={() => toggleStaff(item.staff.id)}
                {disabled}
              />
              <Avatar
                photoUrl={item.staff.photo_url}
                gender={item.staff.gender}
                alt={`Foto de ${item.staff.first_name} ${item.staff.last_name}`}
                size="sm"
              />
              <span class="candidate-info">
                <span class="candidate-name">
                  #{index + 1} · {item.staff.first_name} {item.staff.last_name}
                </span>
                <span class="candidate-role">
                  {labelOf(staffRoleLabels, item.staff.role)}
                </span>
              </span>
            </label>

            {#if checked}
              <button
                type="button"
                class={`lead-toggle ${lead ? 'is-lead' : ''}`}
                onclick={() => setLead(item.staff.id)}
                {disabled}
              >
                {lead ? 'Responsable principal' : 'Marcar responsable'}
              </button>
            {/if}
          </div>

          <div class="reasons-row">
            {#each item.reasons as reason (reason)}
              <span
                class={`reason-pill ${reason.startsWith('Advertencia') ? 'is-warning' : ''}`}
              >
                {reason}
              </span>
            {/each}
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if selectedActiveMembers.length > 0}
    <div
      class="inactive-assigned current-assigned"
      role="group"
      aria-label="Personal ya asignado al servicio"
    >
      <p class="status-note">
        Estos funcionarios ya están asignados al servicio. Si no aparecen en las sugerencias, podés
        gestionarlos acá, quitarlos o cambiar el responsable.
      </p>
      <ul class="candidate-grid">
        {#each selectedActiveMembers as member (member.id)}
          {@const lead = isLead(member.id)}
          <li class="candidate-card is-selected">
            <div class="candidate-main">
              <label class="candidate-check">
                <input
                  type="checkbox"
                  checked
                  onchange={() => toggleStaff(member.id)}
                  {disabled}
                />
                <Avatar
                  photoUrl={member.photo_url}
                  gender={member.gender}
                  alt={`Foto de ${member.first_name} ${member.last_name}`}
                  size="sm"
                />
                <span class="candidate-info">
                  <span class="candidate-name">
                    {member.first_name} {member.last_name}
                  </span>
                  <span class="candidate-role">
                    {labelOf(staffRoleLabels, member.role)} · Asignación actual
                  </span>
                </span>
              </label>

              <button
                type="button"
                class={`lead-toggle ${lead ? 'is-lead' : ''}`}
                onclick={() => setLead(member.id)}
                {disabled}
              >
                {lead ? 'Responsable principal' : 'Marcar responsable'}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if extraActiveStaff.length > 0}
    <div class="manual-override">
      <button
        type="button"
        class="button button-secondary"
        onclick={() => (showAllStaff = !showAllStaff)}
      >
        {showAllStaff
          ? 'Ocultar resto del personal activo'
          : `Mostrar resto del personal activo (${extraActiveStaff.length})`}
      </button>

      {#if showAllStaff}
        <ul class="candidate-grid extra-grid">
          {#each extraActiveStaff as member (member.id)}
            {@const checked = isSelected(member.id)}
            {@const lead = isLead(member.id)}
            <li class={`candidate-card ${checked ? 'is-selected' : ''}`}>
              <div class="candidate-main">
                <label class="candidate-check">
                  <input
                    type="checkbox"
                    {checked}
                    onchange={() => toggleStaff(member.id)}
                    {disabled}
                  />
                  <Avatar
                    photoUrl={member.photo_url}
                    gender={member.gender}
                    alt={`Foto de ${member.first_name} ${member.last_name}`}
                    size="sm"
                  />
                  <span class="candidate-info">
                    <span class="candidate-name">
                      {member.first_name} {member.last_name}
                    </span>
                    <span class="candidate-role">
                      {labelOf(staffRoleLabels, member.role)} · Asignación manual
                    </span>
                  </span>
                </label>

                {#if checked}
                  <button
                    type="button"
                    class={`lead-toggle ${lead ? 'is-lead' : ''}`}
                    onclick={() => setLead(member.id)}
                    {disabled}
                  >
                    {lead ? 'Responsable principal' : 'Marcar responsable'}
                  </button>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  .availability-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .section-header h3 {
    font-size: var(--text-base);
  }

  .section-header p {
    margin-top: 0.15rem;
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .selected-counter {
    padding: 0.25rem 0.65rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .status-note {
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .inactive-assigned {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-md);
    background: var(--color-warning-soft);
  }

  .candidate-card.is-inactive {
    border-color: var(--color-warning);
  }

  .current-assigned {
    border-color: var(--color-line-strong);
    background: var(--color-surface-raised);
  }

  .candidate-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .candidate-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface);
  }

  .candidate-card.is-selected {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px var(--color-primary);
  }

  .candidate-main {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .candidate-check {
    display: flex;
    flex: 1;
    min-width: 0;
    align-items: center;
    gap: var(--space-2);
    cursor: pointer;
  }

  .candidate-info {
    display: flex;
    min-width: 0;
    flex-direction: column;
  }

  .candidate-name {
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .candidate-role {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .lead-toggle {
    padding: 0.3rem 0.6rem;
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .lead-toggle.is-lead {
    border-color: var(--color-primary);
    background: var(--color-primary);
    color: var(--color-on-primary);
  }

  .reasons-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .reason-pill {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
    font-size: 0.7rem;
    font-weight: 600;
  }

  .reason-pill.is-warning {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .manual-override {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    padding-top: var(--space-2);
  }

  .extra-grid {
    width: 100%;
  }
</style>
