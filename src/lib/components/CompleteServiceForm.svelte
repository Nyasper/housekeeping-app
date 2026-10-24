<script lang="ts">
  import { untrack } from 'svelte';

  interface Props {
    formId: string;
    initialStartTime: string;
    initialEndTime: string;
    dirty?: boolean;
    onsubmit: (times: { startTime: string; endTime: string }) => Promise<void> | void;
  }

  let {
    formId,
    initialStartTime,
    initialEndTime,
    dirty = $bindable(false),
    onsubmit
  }: Props = $props();

  const initialTimes = untrack(() => ({
    startTime: initialStartTime,
    endTime: initialEndTime
  }));

  let startTime = $state(initialTimes.startTime);
  let endTime = $state(initialTimes.endTime);

  const isDirty = $derived(
    startTime !== initialTimes.startTime || endTime !== initialTimes.endTime
  );

  $effect(() => {
    dirty = isDirty;
  });

  function handleSubmit(event: SubmitEvent): void {
    event.preventDefault();
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
      <input class="input" type="time" bind:value={startTime} required />
    </label>
    <label>
      <span>Hora de fin</span>
      <input class="input" type="time" bind:value={endTime} required />
    </label>
  </div>
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
</style>
