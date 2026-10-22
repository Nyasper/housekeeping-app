<script lang="ts">
  import { onMount } from 'svelte';

  import {
    labelOf,
    placeStatusLabels,
    placeTypeLabels,
    serviceTypeLabels,
    toOptions
  } from '#lib/api/labels.js';
  import type {
    Place,
    PlaceInput,
    PlaceStatus,
    PlaceType
  } from '#lib/api/types.js';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import EmptyState from '#lib/components/EmptyState.svelte';
  import PlaceForm from '#lib/components/PlaceForm.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { placesService } from '#lib/services/placesService.svelte.js';
  import { formatDate } from '#lib/utils/format.js';

  const typeOptions = toOptions(placeTypeLabels);
  const statusOptions = toOptions(placeStatusLabels);

  let searchInput = $state('');
  let debouncedSearch = $state('');
  let typeFilter = $state<PlaceType | 'all'>('all');
  let statusFilter = $state<PlaceStatus | 'all'>('all');
  let activeFilter = $state<'active' | 'inactive' | 'all'>('active');

  let showForm = $state(false);
  let editingPlace = $state<Place | null>(null);
  let deactivatingPlace = $state<Place | null>(null);

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
    void placesService.load();
  });

  const filteredPlaces = $derived.by(() => {
    return placesService.items.filter((place) => {
      if (activeFilter === 'active' && !place.is_active) {
        return false;
      }
      if (activeFilter === 'inactive' && place.is_active) {
        return false;
      }
      if (typeFilter !== 'all' && place.type !== typeFilter) {
        return false;
      }
      if (statusFilter !== 'all' && place.effective_status !== statusFilter) {
        return false;
      }
      if (debouncedSearch) {
        const name = place.name.toLocaleLowerCase('es-UY');
        const location = (place.location ?? '').toLocaleLowerCase('es-UY');
        if (!name.includes(debouncedSearch) && !location.includes(debouncedSearch)) {
          return false;
        }
      }
      return true;
    });
  });

  const activeApartmentsCount = $derived(
    placesService.items.filter((p) => p.is_active && p.type === 'apartment').length
  );
  const activeCommonAreasCount = $derived(
    placesService.items.filter((p) => p.is_active && p.type === 'common_area').length
  );

  function openCreateForm(): void {
    editingPlace = null;
    showForm = true;
    placesService.error = null;
  }

  function openEditForm(place: Place): void {
    editingPlace = place;
    showForm = true;
    placesService.error = null;
  }

  function closeForm(): void {
    showForm = false;
    editingPlace = null;
    placesService.error = null;
  }

  async function handleFormSubmit(input: PlaceInput): Promise<void> {
    if (editingPlace) {
      const updated = await placesService.update(editingPlace.id, input);
      if (updated) {
        toastTone = 'success';
        toastMessage = 'Lugar actualizado.';
        closeForm();
      }
    } else {
      const created = await placesService.create(input);
      if (created) {
        toastTone = 'success';
        toastMessage = 'Lugar registrado.';
        closeForm();
      }
    }
  }

  async function toggleCancelledStatus(place: Place): Promise<void> {
    const nextStatus: PlaceStatus | null = place.status === 'cancelled' ? null : 'cancelled';
    const updated = await placesService.setStatusOverride(place, nextStatus);
    if (updated) {
      toastTone = 'success';
      toastMessage =
        nextStatus === 'cancelled'
          ? `Turno de limpieza omitido para ${place.name}.`
          : `Estado automático restaurado para ${place.name}.`;
    } else if (placesService.error) {
      toastTone = 'error';
      toastMessage = placesService.error;
    }
  }

  async function confirmDeactivate(): Promise<void> {
    const target = deactivatingPlace;
    if (!target) {
      return;
    }
    const updated = await placesService.deactivate(target.id);
    deactivatingPlace = null;
    if (updated) {
      toastTone = 'success';
      toastMessage = `«${target.name}» fue desactivado.`;
    } else if (placesService.error) {
      toastTone = 'error';
      toastMessage = placesService.error;
    }
  }

  async function handleReactivate(place: Place): Promise<void> {
    const updated = await placesService.reactivate(place.id);
    if (updated) {
      toastTone = 'success';
      toastMessage = `«${place.name}» fue reactivado.`;
    } else if (placesService.error) {
      toastTone = 'error';
      toastMessage = placesService.error;
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || deactivatingPlace !== null) {
      return;
    }
    if (showForm) {
      closeForm();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<svelte:head>
  <title>Lugares · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">
      {activeApartmentsCount} apartamentos · {activeCommonAreasCount} áreas comunes
    </p>
    <h1 class="page-title">Lugares</h1>
    <p class="page-lede">
      Administrá los apartamentos y áreas comunes del edificio, sus fechas de estadía y las tareas
      sugeridas.
    </p>
  </div>
  {#if !showForm}
    <button type="button" class="button button-primary" onclick={openCreateForm}>
      Nuevo lugar
    </button>
  {/if}
</header>

<Toast message={toastMessage} tone={toastTone} ondismiss={() => (toastMessage = null)} />

{#if showForm}
  <PlaceForm
    initial={editingPlace}
    busy={placesService.saving}
    error={placesService.error}
    onsubmit={handleFormSubmit}
    oncancel={closeForm}
  />
{/if}

<section class="filters panel" aria-label="Filtros de lugares">
  <div class="filter-field filter-search">
    <label for="place-search">Buscar por nombre o ubicación</label>
    <input
      id="place-search"
      class="input"
      type="search"
      placeholder="Ej. Apto 101 o Piso 3"
      value={searchInput}
      oninput={handleSearchInput}
    />
  </div>

  <div class="filter-field">
    <label for="place-filter-type">Tipo</label>
    <select id="place-filter-type" class="input" bind:value={typeFilter}>
      <option value="all">Todos los tipos</option>
      {#each typeOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </div>

  <div class="filter-field">
    <label for="place-filter-status">Estado de ocupación</label>
    <select id="place-filter-status" class="input" bind:value={statusFilter}>
      <option value="all">Todos los estados</option>
      {#each statusOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </div>

  <div class="filter-field">
    <label for="place-filter-active">Vigencia</label>
    <select id="place-filter-active" class="input" bind:value={activeFilter}>
      <option value="active">Activos</option>
      <option value="inactive">Desactivados</option>
      <option value="all">Todos</option>
    </select>
  </div>
</section>

{#if placesService.error && !showForm}
  <section class="panel error-banner" role="alert">
    <p>{placesService.error}</p>
    <button type="button" class="button button-secondary" onclick={() => placesService.load()}>
      Reintentar
    </button>
  </section>
{/if}

{#if placesService.loading && placesService.items.length === 0}
  <section class="panel status-panel" role="status">
    <p>Cargando lugares…</p>
  </section>
{:else if filteredPlaces.length === 0}
  <section class="panel status-panel">
    <EmptyState
      title="No se encontraron lugares"
      description={placesService.items.length === 0
        ? 'Registrá los apartamentos y áreas comunes del edificio para comenzar a planificar servicios.'
        : 'Probá ajustar los filtros de búsqueda, tipo o estado.'}
    >
      {#if placesService.items.length === 0 && !showForm}
        <button type="button" class="button button-primary" onclick={openCreateForm}>
          Registrar primer lugar
        </button>
      {/if}
    </EmptyState>
  </section>
{:else}
  <div class="places-grid">
    {#each filteredPlaces as place (place.id)}
      <article
        class={`place-card panel ${!place.is_active ? 'is-inactive' : ''} ${place.effective_status === 'recently_vacated' ? 'is-priority' : ''}`}
      >
        <div class="card-header">
          <div>
            <div class="title-row">
              <h2>{place.name}</h2>
              <span class="badge badge-type">{labelOf(placeTypeLabels, place.type)}</span>
              {#if !place.is_active}
                <span class="badge badge-danger">Desactivado</span>
              {:else if place.type === 'apartment'}
                <span class={`badge badge-status-${place.effective_status}`}>
                  {labelOf(placeStatusLabels, place.effective_status)}
                  {#if place.status}
                    (manual)
                  {/if}
                </span>
              {/if}
              {#if place.is_active && place.recommended_service_type}
                <span class="badge badge-recommended">
                  Sugerencia: {labelOf(serviceTypeLabels, place.recommended_service_type)}
                </span>
              {/if}
            </div>
            {#if place.location}
              <p class="location-line">{place.location}</p>
            {/if}
          </div>
        </div>

        {#if place.type === 'apartment'}
          <dl class="stay-details">
            <div>
              <dt>Ingreso (check-in)</dt>
              <dd>{formatDate(place.check_in_date, 'Sin datos')}</dd>
            </div>
            <div>
              <dt>Egreso (check-out)</dt>
              <dd>{formatDate(place.check_out_date, 'Sin datos')}</dd>
            </div>
            <div>
              <dt>Tarea recomendada hoy</dt>
              <dd>
                {#if place.effective_status === 'cancelled'}
                  Turno omitido (cancelado desde {formatDate(place.updated_at.slice(0, 10))})
                {:else if place.recommended_service_type}
                  {labelOf(serviceTypeLabels, place.recommended_service_type)}
                {:else}
                  Sin recomendación automática
                {/if}
              </dd>
            </div>
          </dl>
        {/if}

        {#if place.notes}
          <p class="card-notes">{place.notes}</p>
        {/if}

        <div class="card-actions">
          <button
            type="button"
            class="button button-secondary"
            onclick={() => openEditForm(place)}
          >
            Editar
          </button>

          {#if place.is_active && place.type === 'apartment'}
            <button
              type="button"
              class="button button-secondary"
              onclick={() => toggleCancelledStatus(place)}
              disabled={placesService.saving}
            >
              {place.status === 'cancelled' ? 'Revertir cancelación' : 'Omitir turno (Cancelar)'}
            </button>
          {/if}

          {#if place.is_active}
            <button
              type="button"
              class="button button-secondary"
              onclick={() => (deactivatingPlace = place)}
            >
              Desactivar
            </button>
          {:else}
            <button
              type="button"
              class="button button-primary"
              onclick={() => handleReactivate(place)}
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
  open={deactivatingPlace !== null}
  title="Desactivar lugar"
  description={deactivatingPlace
    ? `¿Querés desactivar «${deactivatingPlace.name}»? Se conservará todo su historial y podrás reactivarlo cuando vuelva a estar operativo.`
    : ''}
  confirmLabel="Desactivar"
  tone="danger"
  busy={placesService.saving}
  onconfirm={confirmDeactivate}
  oncancel={() => (deactivatingPlace = null)}
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

  .places-grid {
    display: grid;
    gap: var(--space-4);
  }

  .place-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .place-card.is-inactive {
    opacity: 0.78;
  }

  .place-card.is-priority {
    border-color: var(--color-accent);
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

  .location-line {
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

  .badge-recommended {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-danger,
  .badge-status-cancelled {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-status-occupied {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-status-upcoming_occupancy {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .badge-status-recently_vacated {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .badge-status-vacant {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .stay-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: var(--space-3);
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
    border: 1px solid var(--color-line);
  }

  .stay-details dt {
    font-size: var(--text-xs);
    font-weight: 650;
    color: var(--color-ink-muted);
  }

  .stay-details dd {
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

  @media (max-width: 900px) {
    .filters {
      grid-template-columns: 1fr;
    }
  }
</style>
