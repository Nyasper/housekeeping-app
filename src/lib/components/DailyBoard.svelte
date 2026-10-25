<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';

  import { isTauriRuntime } from '#lib/api/invoke.js';
  import {
    absenceTypeLabels,
    incidentTypeLabels,
    labelOf,
    placeStatusLabels,
    serviceStatusLabels,
    serviceTypeLabels,
    staffRoleLabels,
    weekdayLabels
  } from '#lib/api/labels.js';
  import type {
    Absence,
    CleaningService,
    Incident,
    Place,
    ServiceStatus,
    ServiceType,
    Staff
  } from '#lib/api/types.js';
  import {
    durationMinutesBetween,
    formatDate,
    formatDurationMinutes,
    formatTime,
    formatUtcDateTimeDate,
    weekdayFromIsoDate
  } from '#lib/utils/format.js';
  import Avatar from './Avatar.svelte';
  import EmptyState from './EmptyState.svelte';

  interface Props {
    selectedDate: string;
    isToday: boolean;
    places: Place[];
    services: CleaningService[];
    staff: Staff[];
    absences: Absence[];
    openIncidents: Incident[];
    loading?: boolean;
    busy?: boolean;
    onprevday: () => void;
    onnextday: () => void;
    ontoday: () => void;
    ondatechange: (date: string) => void;
    oncreateservice: (preset?: { placeId: number; serviceType: ServiceType }) => void;
    oneditservice: (service: CleaningService) => void;
    onstartservice: (service: CleaningService) => void;
    oncompleteservice: (service: CleaningService) => void;
    oncancelservice: (service: CleaningService) => void;
    ontoggleplacecancelled: (place: Place) => void;
    onaddincident: (serviceId?: number) => void;
    onresolveincident: (incident: Incident) => void;
  }

  let {
    selectedDate,
    isToday,
    places,
    services,
    staff,
    absences,
    openIncidents,
    loading = false,
    busy = false,
    onprevday,
    onnextday,
    ontoday,
    ondatechange,
    oncreateservice,
    oneditservice,
    onstartservice,
    oncompleteservice,
    oncancelservice,
    ontoggleplacecancelled,
    onaddincident,
    onresolveincident
  }: Props = $props();

  let statusFilter = $state<ServiceStatus | 'all'>('all');
  let searchInput = $state('');
  let debouncedSearch = $state('');
  let showUnavailableStaff = $state(true);
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

  // Active (non-cancelled) services for the selected date
  const activeDayServices = $derived(
    services.filter((service) => service.status !== 'cancelled')
  );

  const scheduledPlaceIds = $derived(new Set(activeDayServices.map((s) => s.place_id)));

  // Priority rank for recommended apartments: recently_vacated -> upcoming_occupancy -> occupied
  function statusPriority(status: Place['effective_status']): number {
    switch (status) {
      case 'recently_vacated':
        return 0;
      case 'upcoming_occupancy':
        return 1;
      case 'occupied':
        return 2;
      default:
        return 3;
    }
  }

  // Section 2: Suggested pending apartments (active, has recommendation, not cancelled, no service created for date)
  const suggestedPendingPlaces = $derived.by(() => {
    return places
      .filter(
        (place) =>
          place.is_active &&
          place.type === 'apartment' &&
          place.effective_status !== 'cancelled' &&
          place.recommended_service_type !== null &&
          place.recommended_service_type !== undefined &&
          !scheduledPlaceIds.has(place.id)
      )
      .sort((a, b) => {
        const diff = statusPriority(a.effective_status) - statusPriority(b.effective_status);
        return diff !== 0 ? diff : a.name.localeCompare(b.name, 'es-UY');
      });
  });

  // Apartments with manual 'cancelled' override (turn omitted)
  const cancelledApartments = $derived(
    places.filter(
      (place) =>
        place.is_active &&
        place.type === 'apartment' &&
        place.effective_status === 'cancelled'
    )
  );

  // Staff availability breakdown for `selectedDate`
  function getStaffUnavailabilityReason(member: Staff, date: string): string | null {
    if (!member.is_active) {
      return 'Dado de baja';
    }
    if (member.is_seasonal && member.season_start_date && member.season_end_date) {
      if (date < member.season_start_date || date > member.season_end_date) {
        return `Fuera de temporada (${formatDate(member.season_start_date)} – ${formatDate(member.season_end_date)})`;
      }
    }
    const absence = absences.find(
      (a) => a.staff_id === member.id && a.start_date <= date && a.end_date >= date
    );
    if (absence) {
      return `${labelOf(absenceTypeLabels, absence.type)} (hasta ${formatDate(absence.end_date)})`;
    }
    const weekday = weekdayFromIsoDate(date);
    if (weekday !== null && member.rest_days.includes(weekday)) {
      return `Descanso semanal (${weekdayLabels[weekday]})`;
    }
    return null;
  }

  const activeStaffMembers = $derived(staff.filter((m) => m.is_active));

  const availableStaffList = $derived(
    activeStaffMembers.filter((m) => getStaffUnavailabilityReason(m, selectedDate) === null)
  );

  const unavailableStaffList = $derived(
    activeStaffMembers
      .map((member) => ({
        member,
        reason: getStaffUnavailabilityReason(member, selectedDate)
      }))
      .filter((entry): entry is { member: Staff; reason: string } => entry.reason !== null)
  );

  const unavailableStaffIds = $derived(
    new Set(unavailableStaffList.map((entry) => entry.member.id))
  );

  // Map staff_id -> services assigned on selectedDate (excluding cancelled)
  const assignmentsByStaffId = $derived.by(() => {
    const record: Record<number, CleaningService[]> = {};
    for (const service of activeDayServices) {
      for (const assigned of service.staff) {
        const list = record[assigned.staff_id] ?? [];
        record[assigned.staff_id] = [...list, service];
      }
    }
    return record;
  });

  // Map service_id -> open incidents count
  const openIncidentsByServiceId = $derived.by(() => {
    const record: Record<number, Incident[]> = {};
    for (const incident of openIncidents) {
      const list = record[incident.service_id] ?? [];
      record[incident.service_id] = [...list, incident];
    }
    return record;
  });

  // Service ordering: in_progress -> planned -> completed -> cancelled
  function serviceOrderWeight(status: ServiceStatus): number {
    switch (status) {
      case 'in_progress':
        return 0;
      case 'planned':
        return 1;
      case 'completed':
        return 2;
      case 'cancelled':
        return 3;
    }
  }

  const filteredServices = $derived.by(() => {
    return services
      .filter((service) => {
        if (statusFilter !== 'all' && service.status !== statusFilter) {
          return false;
        }
        if (debouncedSearch) {
          const placeMatch = service.place_name
            .toLocaleLowerCase('es-UY')
            .includes(debouncedSearch);
          const staffMatch = service.staff.some((s) =>
            `${s.first_name} ${s.last_name}`.toLocaleLowerCase('es-UY').includes(debouncedSearch)
          );
          if (!placeMatch && !staffMatch) {
            return false;
          }
        }
        return true;
      })
      .sort((a, b) => {
        const w = serviceOrderWeight(a.status) - serviceOrderWeight(b.status);
        return w !== 0 ? w : a.place_name.localeCompare(b.place_name, 'es-UY');
      });
  });

  // Summary metrics
  const inProgressCount = $derived(
    services.filter((s) => s.status === 'in_progress').length
  );
  const plannedCount = $derived(
    services.filter((s) => s.status === 'planned').length
  );
  const completedCount = $derived(
    services.filter((s) => s.status === 'completed').length
  );
  const recentlyVacatedPendingCount = $derived(
    suggestedPendingPlaces.filter((p) => p.effective_status === 'recently_vacated').length
  );

  function serviceAnomalies(service: CleaningService): string[] {
    if (service.status === 'cancelled') {
      return [];
    }
    const issues: string[] = [];
    const inactiveNames = service.staff
      .filter((s) => s.is_active === false)
      .map((s) => `${s.first_name} ${s.last_name}`);
    if (inactiveNames.length > 0) {
      issues.push(`Personal dado de baja: ${inactiveNames.join(', ')}`);
    }
    if (service.status !== 'completed') {
      const unavailableNames = service.staff
        .filter((s) => s.is_active !== false && unavailableStaffIds.has(s.staff_id))
        .map((s) => `${s.first_name} ${s.last_name}`);
      if (unavailableNames.length > 0) {
        issues.push(`No disponible en la fecha: ${unavailableNames.join(', ')}`);
      }
    }
    return issues;
  }

  function resolveIncidentPhotoSrc(photoUrl: string | null): string | null {
    if (!photoUrl) {
      return null;
    }
    if (
      photoUrl.startsWith('http://') ||
      photoUrl.startsWith('https://') ||
      photoUrl.startsWith('asset:') ||
      photoUrl.startsWith('data:')
    ) {
      return photoUrl;
    }
    if (isTauriRuntime()) {
      return convertFileSrc(photoUrl);
    }
    return null;
  }
</script>

<!-- 1. Date selector & Daily KPIs -->
<section class="command-bar panel" aria-label="Selector de fecha y resumen del día">
  <div class="date-nav">
    <button
      type="button"
      class="button button-secondary"
      onclick={onprevday}
      aria-label="Ir al día anterior"
    >
      ← Anterior
    </button>
    <div class="date-input-wrap">
      <label class="visually-hidden" for="board-date-picker">Fecha del tablero</label>
      <input
        id="board-date-picker"
        class="input date-picker"
        type="date"
        value={selectedDate}
        onchange={(e) => ondatechange((e.currentTarget as HTMLInputElement).value)}
      />
    </div>
    <button
      type="button"
      class="button button-secondary"
      onclick={onnextday}
      aria-label="Ir al día siguiente"
    >
      Siguiente →
    </button>
    {#if !isToday}
      <button type="button" class="button button-primary" onclick={ontoday}>
        Volver a hoy
      </button>
    {:else}
      <span class="today-pill">Hoy</span>
    {/if}
  </div>

  <div class="metrics-strip" role="region" aria-label="Indicadores del día">
    <div class={`metric-tile ${suggestedPendingPlaces.length > 0 ? 'has-attention' : ''}`}>
      <span class="metric-label">Pendientes sugeridos</span>
      <strong class="metric-value">{suggestedPendingPlaces.length}</strong>
      <span class="metric-sub">
        {#if recentlyVacatedPendingCount > 0}
          {recentlyVacatedPendingCount} egreso{recentlyVacatedPendingCount === 1 ? '' : 's'} reciente{recentlyVacatedPendingCount === 1 ? '' : 's'}
        {:else}
          Sin egresos urgentes
        {/if}
      </span>
    </div>

    <div class="metric-tile">
      <span class="metric-label">Servicios activos</span>
      <strong class="metric-value">{inProgressCount + plannedCount}</strong>
      <span class="metric-sub">
        {inProgressCount} en curso · {plannedCount} planificado{plannedCount === 1 ? '' : 's'}
      </span>
    </div>

    <div class="metric-tile">
      <span class="metric-label">Completados</span>
      <strong class="metric-value">{completedCount}</strong>
      <span class="metric-sub">
        de {activeDayServices.length} programado{activeDayServices.length === 1 ? '' : 's'}
      </span>
    </div>

    <div class="metric-tile">
      <span class="metric-label">Personal disponible</span>
      <strong class="metric-value">{availableStaffList.length}</strong>
      <span class="metric-sub">
        de {activeStaffMembers.length} activo{activeStaffMembers.length === 1 ? '' : 's'}
      </span>
    </div>

    <div class={`metric-tile ${openIncidents.length > 0 ? 'has-danger' : ''}`}>
      <span class="metric-label">Incidencias abiertas</span>
      <strong class="metric-value">{openIncidents.length}</strong>
      <span class="metric-sub">
        {openIncidents.length === 0 ? 'Todo al día' : 'Pendientes de resolver'}
      </span>
    </div>
  </div>
</section>

{#if loading && places.length === 0 && services.length === 0}
  <section class="panel loading-panel" role="status">
    <p>Cargando tablero del día…</p>
  </section>
{:else}
  <div class="board-layout">
    <!-- Left / Main Operational Column -->
    <div class="board-main">
      <!-- 2. Suggested Pending Apartments -->
      <section class="board-section panel" aria-labelledby="heading-pending">
        <div class="section-head">
          <div>
            <h2 id="heading-pending">Pendientes sugeridos</h2>
            <p>
              Apartamentos con tarea recomendada según estadía y sin servicio asignado en esta
              fecha.
            </p>
          </div>
          <span class="counter-badge">{suggestedPendingPlaces.length}</span>
        </div>

        {#if suggestedPendingPlaces.length === 0}
          <div class="compact-empty">
            <p class="compact-empty-title">
              No quedan apartamentos con limpieza sugerida pendiente para esta fecha.
            </p>
            <p class="compact-empty-sub">
              Podés programar igualmente un servicio en un área común, vidrios o cualquier
              apartamento.
            </p>
          </div>
        {:else}
          <div class="pending-grid">
            {#each suggestedPendingPlaces as place (place.id)}
              {@const recType = place.recommended_service_type ?? 'daily_cleaning'}
              {@const isUrgent = place.effective_status === 'recently_vacated'}
              <article class={`pending-card ${isUrgent ? 'is-urgent' : ''}`}>
                <div class="pending-top">
                  <div>
                    <div class="title-row">
                      <h3>{place.name}</h3>
                      {#if place.location}
                        <span class="location-tag">{place.location}</span>
                      {/if}
                    </div>
                    <div class="badges-row">
                      <span class={`badge badge-place-${place.effective_status}`}>
                        {labelOf(placeStatusLabels, place.effective_status)}
                      </span>
                      <span class="badge badge-task">
                        Tarea sugerida: {labelOf(serviceTypeLabels, recType)}
                      </span>
                    </div>
                  </div>
                </div>

                <p class="stay-line">
                  Ingreso: {formatDate(place.check_in_date, 'Sin datos')} · Egreso: {formatDate(
                    place.check_out_date,
                    'Sin datos'
                  )}
                </p>

                <div class="pending-actions">
                  <button
                    type="button"
                    class="button button-primary button-sm"
                    onclick={() =>
                      oncreateservice({
                        placeId: place.id,
                        serviceType: recType
                      })}
                    disabled={busy}
                  >
                    Programar {labelOf(serviceTypeLabels, recType).toLocaleLowerCase('es-UY')}
                  </button>
                  <button
                    type="button"
                    class="button button-secondary button-sm"
                    onclick={() => ontoggleplacecancelled(place)}
                    disabled={busy}
                  >
                    Omitir turno
                  </button>
                </div>
              </article>
            {/each}
          </div>
        {/if}

        {#if cancelledApartments.length > 0}
          <div class="cancelled-strip" aria-label="Apartamentos con turno omitido">
            <div class="cancelled-header">
              <span class="badge badge-place-cancelled">
                Turnos omitidos ({cancelledApartments.length})
              </span>
              <span class="cancelled-hint">
                Marcados como cancelados manualmente; no generan sugerencia hasta revertirlos.
              </span>
            </div>
            <ul class="cancelled-list">
              {#each cancelledApartments as apt (apt.id)}
                <li class="cancelled-item">
                  <div>
                    <strong>{apt.name}</strong>
                    <span class="cancelled-date">
                      · Cancelado desde {formatUtcDateTimeDate(apt.cancelled_at, 'fecha desconocida')}
                    </span>
                  </div>
                  <button
                    type="button"
                    class="button button-secondary button-sm"
                    onclick={() => ontoggleplacecancelled(apt)}
                    disabled={busy}
                  >
                    Revertir cancelación
                  </button>
                </li>
              {/each}
            </ul>
          </div>
        {/if}
      </section>

      <!-- 3. Services of the Day -->
      <section class="board-section panel" aria-labelledby="heading-services">
        <div class="section-head">
          <div>
            <h2 id="heading-services">Servicios del día</h2>
            <p>Seguimiento de limpiezas programadas, en curso y completadas.</p>
          </div>
          <button
            type="button"
            class="button button-primary button-sm"
            onclick={() => oncreateservice()}
            disabled={busy}
          >
            + Nuevo servicio
          </button>
        </div>

        <div class="services-toolbar">
          <div class="filter-pills" role="group" aria-label="Filtrar servicios por estado">
            <button
              type="button"
              class={`filter-pill ${statusFilter === 'all' ? 'is-active' : ''}`}
              onclick={() => (statusFilter = 'all')}
            >
              Todos ({services.length})
            </button>
            <button
              type="button"
              class={`filter-pill ${statusFilter === 'in_progress' ? 'is-active' : ''}`}
              onclick={() => (statusFilter = 'in_progress')}
            >
              En progreso ({inProgressCount})
            </button>
            <button
              type="button"
              class={`filter-pill ${statusFilter === 'planned' ? 'is-active' : ''}`}
              onclick={() => (statusFilter = 'planned')}
            >
              Planificados ({plannedCount})
            </button>
            <button
              type="button"
              class={`filter-pill ${statusFilter === 'completed' ? 'is-active' : ''}`}
              onclick={() => (statusFilter = 'completed')}
            >
              Completados ({completedCount})
            </button>
          </div>

          <div class="service-search-box">
            <label class="visually-hidden" for="board-service-search">
              Buscar servicio por lugar o personal
            </label>
            <input
              id="board-service-search"
              class="input input-sm"
              type="search"
              placeholder="Buscar lugar o funcionario…"
              value={searchInput}
              oninput={handleSearchInput}
            />
          </div>
        </div>

        {#if filteredServices.length === 0}
          <EmptyState
            title={services.length === 0
              ? 'Todavía no hay servicios cargados para esta fecha'
              : 'Ningún servicio coincide con el filtro actual'}
            description={services.length === 0
              ? 'Programá un servicio desde los pendientes sugeridos de arriba o creá uno nuevo eligiendo lugar y personal.'
              : 'Probá cambiar el estado seleccionado o limpiar el texto de búsqueda.'}
          >
            {#if services.length === 0}
              <button
                type="button"
                class="button button-primary"
                onclick={() => oncreateservice()}
              >
                Crear primer servicio del día
              </button>
            {/if}
          </EmptyState>
        {:else}
          <div class="services-list">
            {#each filteredServices as service (service.id)}
              {@const duration = durationMinutesBetween(service.start_time, service.end_time)}
              {@const anomalies = serviceAnomalies(service)}
              {@const serviceIncidents = openIncidentsByServiceId[service.id] ?? []}
              <article
                class={`service-card ${anomalies.length > 0 ? 'has-anomaly' : ''} status-${service.status}`}
              >
                <div class="service-top">
                  <div>
                    <div class="title-row">
                      <h3>{service.place_name}</h3>
                      <span class="badge badge-type">
                        {labelOf(serviceTypeLabels, service.service_type)}
                      </span>
                      <span class={`badge badge-service-${service.status}`}>
                        {labelOf(serviceStatusLabels, service.status)}
                      </span>
                      {#if serviceIncidents.length > 0}
                        <span class="badge badge-incident">
                          {serviceIncidents.length} incidencia{serviceIncidents.length === 1
                            ? ''
                            : 's'} abierta{serviceIncidents.length === 1 ? '' : 's'}
                        </span>
                      {/if}
                    </div>
                    <p class="service-time">
                      {#if service.start_time && service.end_time}
                        Horario: {formatTime(service.start_time)} a {formatTime(service.end_time)}
                        {#if duration !== null}
                          · <strong>{formatDurationMinutes(duration)}</strong>
                        {/if}
                      {:else if service.start_time}
                        Iniciado a las {formatTime(service.start_time)} · En progreso
                      {:else}
                        Horario aún sin registrar
                      {/if}
                    </p>
                  </div>
                </div>

                {#if anomalies.length > 0}
                  <div class="anomaly-banner" role="status">
                    {#each anomalies as issue (issue)}
                      <p>⚠ Revisar asignación: {issue}</p>
                    {/each}
                  </div>
                {/if}

                <div class="assigned-row" aria-label="Personal asignado">
                  {#each service.staff as assigned (assigned.staff_id)}
                    <div
                      class={`staff-pill ${assigned.is_active === false ? 'is-inactive' : ''}`}
                    >
                      <Avatar
                        photoUrl={assigned.photo_url}
                        gender={assigned.gender}
                        alt={`Foto de ${assigned.first_name} ${assigned.last_name}`}
                        size="sm"
                      />
                      <span>{assigned.first_name} {assigned.last_name}</span>
                      {#if assigned.is_lead}
                        <span class="lead-tag">Responsable</span>
                      {/if}
                    </div>
                  {/each}
                </div>

                {#if service.notes}
                  <p class="service-notes">{service.notes}</p>
                {/if}

                <div class="service-actions">
                  {#if service.status === 'planned'}
                    <button
                      type="button"
                      class="button button-primary button-sm"
                      onclick={() => onstartservice(service)}
                      disabled={busy}
                    >
                      Iniciar ahora
                    </button>
                  {/if}

                  {#if service.status === 'planned' || service.status === 'in_progress'}
                    <button
                      type="button"
                      class="button button-primary button-sm"
                      onclick={() => oncompleteservice(service)}
                      disabled={busy}
                    >
                      Completar
                    </button>
                  {/if}

                  {#if service.status !== 'cancelled'}
                    <button
                      type="button"
                      class="button button-secondary button-sm"
                      onclick={() => onaddincident(service.id)}
                      disabled={busy}
                    >
                      + Incidencia
                    </button>
                  {/if}

                  <button
                    type="button"
                    class="button button-secondary button-sm"
                    onclick={() => oneditservice(service)}
                    disabled={busy}
                  >
                    {service.status === 'completed' ? 'Editar / Reabrir' : 'Editar'}
                  </button>

                  {#if service.status === 'planned' || service.status === 'in_progress'}
                    <button
                      type="button"
                      class="button button-secondary button-sm"
                      onclick={() => oncancelservice(service)}
                      disabled={busy}
                    >
                      Cancelar
                    </button>
                  {/if}
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <!-- Right / Operational Rail (Staff & Open Incidents) -->
    <aside class="board-rail" aria-label="Personal del día e incidencias abiertas">
      <!-- 4. Staff Availability & Assignments -->
      <section class="board-section panel" aria-labelledby="heading-staff">
        <div class="section-head">
          <div>
            <h2 id="heading-staff">Personal en la fecha</h2>
            <p>
              {availableStaffList.length} disponible{availableStaffList.length === 1 ? '' : 's'} · {unavailableStaffList.length}
              con descanso, ausencia o fuera de temporada.
            </p>
          </div>
        </div>

        {#if activeStaffMembers.length === 0}
          <div class="compact-empty">
            <p class="compact-empty-title">No hay personal activo registrado.</p>
            <p class="compact-empty-sub">Agregá funcionarios en la sección Personal.</p>
          </div>
        {:else}
          <div class="staff-roster">
            {#each availableStaffList as member (member.id)}
              {@const memberServices = assignmentsByStaffId[member.id] ?? []}
              <div class="roster-card">
                <div class="roster-person">
                  <Avatar
                    photoUrl={member.photo_url}
                    gender={member.gender}
                    alt={`Foto de ${member.first_name} ${member.last_name}`}
                    size="sm"
                  />
                  <div class="roster-meta">
                    <div class="roster-name-row">
                      <strong>{member.first_name} {member.last_name}</strong>
                      <span class="role-tag">{labelOf(staffRoleLabels, member.role)}</span>
                      {#if member.is_seasonal}
                        <span class="seasonal-tag">
                          Zafral
                          {#if member.season_start_date && member.season_end_date}
                            · {formatDate(member.season_start_date)} al {formatDate(
                              member.season_end_date
                            )}
                          {/if}
                        </span>
                      {/if}
                    </div>
                    {#if member.main_places.length > 0}
                      <p class="roster-places">
                        Principales: {member.main_places.map((p) => p.name).join(', ')}
                      </p>
                    {/if}
                  </div>
                </div>

                <div class="roster-assignments">
                  {#if memberServices.length === 0}
                    <span class="assignment-free">Disponible · Sin asignaciones aún</span>
                  {:else}
                    {#each memberServices as assignedSvc (assignedSvc.id)}
                      <span class={`assignment-chip status-${assignedSvc.status}`}>
                        {assignedSvc.place_name} ({labelOf(
                          serviceStatusLabels,
                          assignedSvc.status
                        )})
                      </span>
                    {/each}
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          {#if unavailableStaffList.length > 0}
            <div class="unavailable-block">
              <button
                type="button"
                class="unavailable-toggle"
                onclick={() => (showUnavailableStaff = !showUnavailableStaff)}
                aria-expanded={showUnavailableStaff}
              >
                <span>No disponibles en esta fecha ({unavailableStaffList.length})</span>
                <span>{showUnavailableStaff ? 'Ocultar' : 'Mostrar'}</span>
              </button>

              {#if showUnavailableStaff}
                <ul class="unavailable-list">
                  {#each unavailableStaffList as item (item.member.id)}
                    <li class="unavailable-item">
                      <Avatar
                        photoUrl={item.member.photo_url}
                        gender={item.member.gender}
                        alt={`Foto de ${item.member.first_name} ${item.member.last_name}`}
                        size="sm"
                      />
                      <div class="unavailable-info">
                        <strong>{item.member.first_name} {item.member.last_name}</strong>
                        <span>{item.reason}</span>
                      </div>
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        {/if}
      </section>

      <!-- 5. Open Incidents -->
      <section class="board-section panel" aria-labelledby="heading-incidents">
        <div class="section-head">
          <div>
            <h2 id="heading-incidents">Incidencias abiertas</h2>
            <p>Daños, objetos olvidados o faltantes pendientes de resolución.</p>
          </div>
          <button
            type="button"
            class="button button-secondary button-sm"
            onclick={() => onaddincident()}
            disabled={busy || activeDayServices.length === 0}
            title={activeDayServices.length === 0
              ? 'Creá primero un servicio para asociar la incidencia'
              : 'Registrar nueva incidencia'}
          >
            + Agregar
          </button>
        </div>

        {#if openIncidents.length === 0}
          <div class="compact-empty">
            <p class="compact-empty-title">No hay incidencias abiertas.</p>
            <p class="compact-empty-sub">
              Cuando registres un daño, objeto olvidado o faltante de insumos, aparecerá acá hasta
              que se resuelva.
            </p>
          </div>
        {:else}
          <ul class="incidents-list">
            {#each openIncidents as incident (incident.id)}
              {@const photoSrc = resolveIncidentPhotoSrc(incident.photo_url)}
              <li class="incident-card">
                <div class="incident-top">
                  <div>
                    <div class="title-row">
                      <strong>{incident.place_name}</strong>
                      <span class={`badge badge-incident-${incident.type}`}>
                        {labelOf(incidentTypeLabels, incident.type)}
                      </span>
                    </div>
                    <span class="incident-meta">
                      {labelOf(serviceTypeLabels, incident.service_type)} · {formatDate(
                        incident.date
                      )}
                    </span>
                  </div>
                  <button
                    type="button"
                    class="button button-secondary button-sm"
                    onclick={() => onresolveincident(incident)}
                    disabled={busy}
                  >
                    Marcar resuelta
                  </button>
                </div>

                <p class="incident-desc">{incident.description}</p>

                {#if photoSrc}
                  <img
                    class="incident-thumb"
                    src={photoSrc}
                    alt={`Foto de incidencia en ${incident.place_name}`}
                    loading="lazy"
                  />
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </aside>
  </div>
{/if}

<style>
  .command-bar {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    margin-bottom: var(--space-5);
  }

  .date-nav {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-line);
  }

  .date-picker {
    width: auto;
    font-weight: 600;
  }

  .today-pill {
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .metrics-strip {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10.5rem, 1fr));
    gap: var(--space-3);
  }

  .metric-tile {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .metric-tile.has-attention {
    border-color: var(--color-accent);
    background: var(--color-accent-soft);
  }

  .metric-tile.has-danger {
    border-color: var(--color-danger);
    background: var(--color-danger-soft);
  }

  .metric-label {
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .metric-value {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    line-height: 1.1;
  }

  .metric-sub {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .loading-panel {
    padding: var(--space-6);
  }

  .board-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.55fr) minmax(20rem, 1fr);
    align-items: start;
    gap: var(--space-5);
  }

  .board-main,
  .board-rail {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    min-width: 0;
  }

  .board-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .section-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .section-head h2 {
    font-size: var(--text-xl);
  }

  .section-head p {
    margin-top: 0.2rem;
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .counter-badge {
    display: inline-flex;
    min-width: 2rem;
    height: 2rem;
    align-items: center;
    justify-content: center;
    padding: 0 0.6rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-size: var(--text-sm);
    font-weight: 700;
  }

  .compact-empty {
    padding: var(--space-4);
    border: 1px dashed var(--color-line-strong);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .compact-empty-title {
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .compact-empty-sub {
    margin-top: 0.25rem;
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .pending-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16.5rem, 1fr));
    gap: var(--space-3);
  }

  .pending-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .pending-card.is-urgent {
    border-width: 1.5px;
    border-color: var(--color-accent);
    background:
      linear-gradient(145deg, var(--color-accent-soft), var(--color-surface-raised) 58%),
      var(--color-surface-raised);
  }

  .title-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .title-row h3 {
    font-size: var(--text-lg);
  }

  .service-card .title-row h3 {
    font-size: var(--text-base);
  }

  .location-tag {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .badges-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: var(--space-2);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.6rem;
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .badge-type {
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
  }

  .badge-task {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-place-recently_vacated {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .badge-place-upcoming_occupancy {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .badge-place-occupied {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-place-vacant {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .badge-place-cancelled {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-service-planned {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .badge-service-in_progress {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .badge-service-completed {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .badge-service-cancelled {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-incident,
  .badge-incident-damage {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .badge-incident-lost_item {
    background: var(--color-warning-soft);
    color: var(--color-warning);
  }

  .badge-incident-supply_shortage {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .stay-line {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .pending-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-line);
  }

  .button-sm {
    min-height: 2.1rem;
    padding: 0.4rem 0.65rem;
    font-size: var(--text-xs);
  }

  .cancelled-strip {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
  }

  .cancelled-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .cancelled-hint {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .cancelled-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .cancelled-item {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    font-size: var(--text-sm);
  }

  .cancelled-date {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .services-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-line);
  }

  .filter-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .filter-pill {
    min-height: 2.15rem;
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-pill);
    background: var(--color-surface-raised);
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .filter-pill.is-active {
    border-color: var(--color-primary);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .service-search-box {
    flex: 1;
    min-width: 13rem;
    max-width: 18rem;
  }

  .input-sm {
    min-height: 2.25rem;
    padding: 0.4rem 0.7rem;
    font-size: var(--text-sm);
  }

  .services-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(20rem, 100%), 1fr));
    gap: var(--space-3);
  }

  .service-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .service-card.status-in_progress {
    border-left: 4px solid var(--color-accent);
  }

  .service-card.status-completed {
    border-left: 4px solid var(--color-success);
  }

  .service-card.has-anomaly {
    border-color: var(--color-warning);
  }

  .service-time {
    margin-top: 0.2rem;
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .anomaly-banner {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-md);
    background: var(--color-warning-soft);
    color: var(--color-ink);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .assigned-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .staff-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.25rem 0.65rem 0.25rem 0.3rem;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .staff-pill.is-inactive {
    border-color: var(--color-warning);
    background: var(--color-warning-soft);
  }

  .lead-tag {
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-size: 0.68rem;
    font-weight: 650;
  }

  .service-notes {
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
    overflow-wrap: anywhere;
  }

  .service-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-line);
  }

  .staff-roster {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .roster-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
  }

  .roster-person {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .roster-meta {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: 0.15rem;
  }

  .roster-name-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
  }

  .role-tag {
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-muted);
    color: var(--color-ink-soft);
    font-size: 0.7rem;
    font-weight: 600;
  }

  .seasonal-tag {
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-size: 0.7rem;
    font-weight: 650;
  }

  .roster-places {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .roster-assignments {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .assignment-free {
    color: var(--color-success);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .assignment-chip {
    padding: 0.15rem 0.55rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
    font-size: 0.72rem;
    font-weight: 650;
  }

  .assignment-chip.status-in_progress {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .assignment-chip.status-completed {
    background: var(--color-success-soft);
    color: var(--color-success);
  }

  .unavailable-block {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-line);
  }

  .unavailable-toggle {
    display: flex;
    width: 100%;
    min-height: 2rem;
    align-items: center;
    justify-content: space-between;
    padding: 0.25rem 0;
    border: 0;
    background: transparent;
    color: var(--color-ink-soft);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .unavailable-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .unavailable-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    background: var(--color-surface-muted);
    font-size: var(--text-xs);
  }

  .unavailable-info {
    display: flex;
    flex-direction: column;
  }

  .unavailable-info span {
    color: var(--color-ink-muted);
  }

  .incidents-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .incident-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
  }

  .incident-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .incident-meta {
    display: block;
    margin-top: 0.15rem;
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
  }

  .incident-desc {
    color: var(--color-ink);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .incident-thumb {
    width: 100%;
    max-height: 9rem;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-sm);
    object-fit: cover;
  }

  @media (max-width: 1080px) {
    .board-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
