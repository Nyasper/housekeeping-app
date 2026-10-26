<script lang="ts">
  import { onMount } from 'svelte';

  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { backupService } from '#lib/services/backupService.svelte.js';
  import { DEFAULT_SETTINGS, settingsService } from '#lib/services/settingsService.svelte.js';
  import { formatNumber } from '#lib/utils/format.js';

  const MIN_DAYS = 0;
  const MAX_DAYS = 30;

  let recentVacatedDays = $state(DEFAULT_SETTINGS.recent_vacated_days);
  let upcomingOccupancyDays = $state(DEFAULT_SETTINGS.upcoming_occupancy_days);
  let loaded = $state(false);
  let submitted = $state(false);
  let successMessage = $state<string | null>(null);

  let pendingRestorePath = $state<string | null>(null);
  let toastMessage = $state<string | null>(null);
  let toastTone = $state<'success' | 'warning' | 'error'>('success');

  const busy = $derived(settingsService.loading || settingsService.saving || backupService.busy);
  const serviceError = $derived(settingsService.error);
  const recentInvalid = $derived(!isValidDays(recentVacatedDays));
  const upcomingInvalid = $derived(!isValidDays(upcomingOccupancyDays));

  const pendingRestoreFileName = $derived.by(() => {
    if (!pendingRestorePath) {
      return '';
    }
    const parts = pendingRestorePath.split(/[/\\]/);
    return parts[parts.length - 1] ?? pendingRestorePath;
  });

  function isValidDays(value: number): boolean {
    return Number.isInteger(value) && value >= MIN_DAYS && value <= MAX_DAYS;
  }

  function syncFromService(): void {
    recentVacatedDays = settingsService.settings.recent_vacated_days;
    upcomingOccupancyDays = settingsService.settings.upcoming_occupancy_days;
  }

  async function loadSettings(): Promise<void> {
    submitted = false;
    successMessage = null;
    await settingsService.load();

    if (!settingsService.error) {
      syncFromService();
      loaded = true;
    }
  }

  onMount(() => {
    void loadSettings();
  });

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    submitted = true;
    successMessage = null;

    if (recentInvalid) {
      document.getElementById('recent-vacated-days')?.focus();
      return;
    }

    if (upcomingInvalid) {
      document.getElementById('upcoming-occupancy-days')?.focus();
      return;
    }

    const saved = await settingsService.save({
      recent_vacated_days: recentVacatedDays,
      upcoming_occupancy_days: upcomingOccupancyDays
    });

    if (saved) {
      syncFromService();
      successMessage = 'Cambios guardados.';
    }
  }

  function clearSuccess(): void {
    successMessage = null;
  }

  function resetDefaults(): void {
    recentVacatedDays = DEFAULT_SETTINGS.recent_vacated_days;
    upcomingOccupancyDays = DEFAULT_SETTINGS.upcoming_occupancy_days;
    submitted = false;
    successMessage = null;
  }

  async function handleCreateBackup(): Promise<void> {
    toastMessage = null;
    backupService.clearError();

    const savedPath = await backupService.createBackup();
    if (savedPath) {
      toastTone = 'success';
      toastMessage = `Respaldo completo guardado en «${savedPath}».`;
    } else if (backupService.error) {
      toastTone = 'error';
      toastMessage = backupService.error;
    }
  }

  async function handleStartRestore(): Promise<void> {
    toastMessage = null;
    backupService.clearError();

    const selected = await backupService.pickRestoreFile();
    if (selected) {
      pendingRestorePath = selected;
    } else if (backupService.error) {
      toastTone = 'error';
      toastMessage = backupService.error;
    }
  }

  async function handleConfirmRestore(): Promise<void> {
    if (!pendingRestorePath) {
      return;
    }
    const targetPath = pendingRestorePath;
    const manifest = await backupService.restoreBackup(targetPath);
    pendingRestorePath = null;

    if (manifest) {
      syncFromService();
      loaded = true;
      if (backupService.error) {
        toastTone = 'warning';
        toastMessage = backupService.error;
      } else {
        toastTone = 'success';
        toastMessage = `Respaldo restaurado correctamente (${formatNumber(manifest.counts.staff)} funcionarios, ${formatNumber(manifest.counts.places)} lugares, ${formatNumber(manifest.counts.cleaning_services)} servicios, ${formatNumber(manifest.counts.incidents)} incidencias).`;
      }
    } else if (backupService.error) {
      toastTone = 'error';
      toastMessage = backupService.error;
    }
  }

  function handleCancelRestore(): void {
    if (!backupService.restoring) {
      pendingRestorePath = null;
    }
  }
</script>

<svelte:head>
  <title>Ajustes · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">Configuración global y copias de seguridad</p>
    <h1 class="page-title">Ajustes</h1>
    <p class="page-lede">
      Definí las ventanas de recomendación del tablero diario y administrá los respaldos completos
      de la base de datos y las fotografías.
    </p>
  </div>
</header>

<Toast
  message={toastMessage}
  tone={toastTone}
  ondismiss={() => {
    toastMessage = null;
  }}
/>

<div class="settings-grid">
  <form class="settings-form panel" onsubmit={handleSubmit} aria-busy={busy}>
    <div class="form-intro">
      <h2>Ventanas de recomendación</h2>
      <p>
        Se usan para calcular la tarea sugerida de cada apartamento. Podés ajustarlas cuando cambie
        el ritmo de ocupación del edificio.
      </p>
    </div>

    <div class="fields">
      <div class="field">
        <label for="recent-vacated-days">Días después del egreso</label>
        <div class="input-row">
          <input
            id="recent-vacated-days"
            class="input"
            type="number"
            min={MIN_DAYS}
            max={MAX_DAYS}
            step="1"
            bind:value={recentVacatedDays}
            oninput={clearSuccess}
            disabled={busy || !loaded}
            aria-required="true"
            aria-invalid={submitted && recentInvalid}
            aria-describedby={`recent-vacated-hint${submitted && recentInvalid ? ' recent-vacated-error' : ''}`}
          />
          <span class="unit">días</span>
        </div>
        <p id="recent-vacated-hint" class="hint">
          Durante esos días, un apartamento desocupado se sugiere para una limpieza general.
        </p>
        {#if submitted && recentInvalid}
          <p id="recent-vacated-error" class="field-error">
            Ingresá un número entero entre {MIN_DAYS} y {MAX_DAYS}.
          </p>
        {/if}
      </div>

      <div class="field">
        <label for="upcoming-occupancy-days">Días antes del próximo ingreso</label>
        <div class="input-row">
          <input
            id="upcoming-occupancy-days"
            class="input"
            type="number"
            min={MIN_DAYS}
            max={MAX_DAYS}
            step="1"
            bind:value={upcomingOccupancyDays}
            oninput={clearSuccess}
            disabled={busy || !loaded}
            aria-required="true"
            aria-invalid={submitted && upcomingInvalid}
            aria-describedby={`upcoming-occupancy-hint${submitted && upcomingInvalid ? ' upcoming-occupancy-error' : ''}`}
          />
          <span class="unit">días</span>
        </div>
        <p id="upcoming-occupancy-hint" class="hint">
          Durante esos días previos, el tablero sugiere un repaso para dejar el apartamento listo.
        </p>
        {#if submitted && upcomingInvalid}
          <p id="upcoming-occupancy-error" class="field-error">
            Ingresá un número entero entre {MIN_DAYS} y {MAX_DAYS}.
          </p>
        {/if}
      </div>
    </div>

    {#if !loaded && settingsService.loading}
      <p class="loading-note" role="status">Cargando configuración…</p>
    {/if}

    {#if serviceError}
      <div class="feedback feedback-error" role="alert">
        <p>{serviceError}</p>
        {#if !loaded}
          <button
            type="button"
            class="button button-secondary"
            onclick={loadSettings}
            disabled={busy}
          >
            Reintentar carga
          </button>
        {/if}
      </div>
    {/if}

    {#if successMessage}
      <p class="feedback feedback-success" role="status">{successMessage}</p>
    {/if}

    <div class="form-actions">
      <button type="submit" class="button button-primary" disabled={busy || !loaded}>
        {settingsService.saving ? 'Guardando…' : 'Guardar cambios'}
      </button>
      <button type="button" class="button button-secondary" onclick={resetDefaults} disabled={busy}>
        Restablecer valores
      </button>
    </div>
  </form>

  <section class="backup-panel panel" aria-labelledby="backup-title" aria-busy={backupService.busy}>
    <div class="form-intro">
      <h2 id="backup-title">Respaldo y restauración</h2>
      <p>
        Exportá una copia completa en formato ZIP (base de datos consistente + fotografías +
        manifiesto) o restaurá un respaldo anterior de manera segura.
      </p>
    </div>

    <div class="backup-cards">
      <article class="backup-card">
        <div>
          <h3>Crear respaldo completo (ZIP)</h3>
          <p>
            Genera una instantánea íntegra sin detener la aplicación. Incluye el personal, lugares,
            servicios, incidencias, configuración y todas las fotos adjuntas.
          </p>
        </div>
        <div class="backup-card-actions">
          <button
            type="button"
            class="button button-primary"
            onclick={handleCreateBackup}
            disabled={busy}
          >
            {backupService.creating ? 'Creando respaldo…' : 'Guardar respaldo ZIP'}
          </button>
        </div>
      </article>

      <article class="backup-card backup-card-restore">
        <div>
          <h3>Restaurar desde un respaldo</h3>
          <p>
            Verifica la integridad del archivo ZIP y de la base de datos antes de reemplazar los
            datos actuales. Si ocurre algún problema, se recupera automáticamente el estado previo.
          </p>
        </div>
        <div class="backup-card-actions">
          <button
            type="button"
            class="button button-secondary"
            onclick={handleStartRestore}
            disabled={busy}
          >
            {backupService.restoring ? 'Restaurando…' : 'Seleccionar respaldo ZIP…'}
          </button>
        </div>
      </article>
    </div>
  </section>
</div>

<ConfirmDialog
  open={pendingRestorePath !== null}
  title="¿Restaurar este respaldo?"
  description={`Se reemplazarán los datos y fotografías actuales por el contenido de «${pendingRestoreFileName}». Antes de aplicar el cambio se creará una copia de seguridad automática del estado actual.`}
  confirmLabel={backupService.restoring ? 'Restaurando…' : 'Sí, restaurar respaldo'}
  cancelLabel="Cancelar"
  tone="danger"
  busy={backupService.restoring}
  onconfirm={handleConfirmRestore}
  oncancel={handleCancelRestore}
/>

<style>
  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(22rem, 1fr));
    gap: var(--space-5);
    align-items: start;
  }

  .settings-form,
  .backup-panel {
    padding: var(--space-6);
  }

  .form-intro {
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-line);
  }

  .form-intro h2 {
    font-size: var(--text-xl);
  }

  .form-intro p {
    max-width: 56ch;
    margin-top: var(--space-2);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .fields {
    display: grid;
    gap: var(--space-5);
    padding: var(--space-5) 0;
  }

  .field label {
    display: block;
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .input-row .input {
    width: 7rem;
  }

  .unit {
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .hint {
    max-width: 54ch;
    margin-top: var(--space-2);
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    line-height: 1.45;
  }

  .field-error {
    margin-top: var(--space-2);
    color: var(--color-danger);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .loading-note {
    margin-bottom: var(--space-4);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  .feedback {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .feedback-error {
    border: 1px solid var(--color-danger);
    background: var(--color-danger-soft);
    color: var(--color-ink);
  }

  .feedback-success {
    border: 1px solid var(--color-success);
    background: var(--color-success-soft);
    color: var(--color-ink);
  }

  .form-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    padding-top: var(--space-5);
    border-top: 1px solid var(--color-line);
  }

  .backup-cards {
    display: grid;
    gap: var(--space-4);
    padding-top: var(--space-5);
  }

  .backup-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--color-line);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }

  .backup-card-restore {
    border-color: var(--color-line-strong);
  }

  .backup-card h3 {
    font-size: var(--text-base);
    font-weight: 650;
  }

  .backup-card p {
    margin-top: var(--space-2);
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    line-height: 1.5;
  }

  .backup-card-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  @media (max-width: 600px) {
    .settings-form,
    .backup-panel {
      padding: var(--space-5) var(--space-4);
    }

    .form-actions {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
