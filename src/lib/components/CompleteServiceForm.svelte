<script lang="ts">
  import { untrack } from 'svelte';

  interface Props {
    formId: string;
    initialStartTime: string;
    initialEndTime: string;
    error?: string | null;
    dirty?: boolean;
    onsubmit: (times: { startTime: string; endTime: string }) => Promise<void> | void;
  }

  let {
    formId,
    initialStartTime,
    initialEndTime,
    error = null,
    dirty = $bindable(false),
    onsubmit
  }: Props = $props();

  const errorId = $props.id();

  const initialTimes = untrack(() => ({
    startTime: initialStartTime,
    endTime: initialEndTime
  }));

  let startTime = $state(initialTimes.startTime);
  let endTime = $state(initialTimes.endTime);

  const isDirty = $derived(
    startTime !== initialTimes.startTime || endTime !== initialTimes.endTime
  );

  const timeError = $derived.by(() => {
    if (!startTime || !endTime) {
      return 'Indicá la hora de inicio y la de finalización.';
    }
    if (startTime >= endTime) {
      return 'La hora de inicio debe ser anterior a la hora de finalización.';
    }
    return null;
  });

  const displayedError = $derived(timeError ?? error);

  $effect(() => {
    dirty = isDirty;
  });

  function handleSubmit(event: SubmitEvent): void {
    event.preventDefault();
    if (timeError) {
      return;
    }
    void onsubmit({ startTime, endTime });
  }
</script>

<form id={formId} class="complete-form" onsubmit={handleSubmit}>
  <p class="complete-hint">
    Confirmá la hora de inicio y de finalización para calcular la duración real del servicio.
  </p>
  <div class="complete-times">
    <label>
      <span>Hora de inicio</span>
      <input
        class="input"
        type="time"
        bind:value={startTime}
        required
        aria-invalid={Boolean(displayedError)}
        aria-describedby={displayedError ? errorId : undefined}
      />
    </label>
    <label>
      <span>Hora de fin</span>
      <input
        class="input"
        type="time"
        bind:value={endTime}
        required
        aria-invalid={Boolean(displayedError)}
        aria-describedby={displayedError ? errorId : undefined}
      />
    </label>
  </div>
  {#if displayedError}
    <p id={errorId} class="form-alert" role="alert">{displayedError}</p>
  {/if}
</form>

<style>
  .complete-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .complete-hint {
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .complete-times {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
  }

  .complete-times label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
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
