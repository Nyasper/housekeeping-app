<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    id: string;
    label: string;
    required?: boolean;
    hint?: string;
    error?: string | null;
    children: Snippet;
  }

  let { id, label, required = false, hint, error = null, children }: Props = $props();

  let fieldElement = $state<HTMLDivElement | null>(null);

  $effect(() => {
    const control = fieldElement?.querySelector<HTMLElement>('input, select, textarea');
    if (!control) {
      return;
    }

    control.setAttribute('aria-required', required ? 'true' : 'false');

    const describedBy = [hint ? `${id}-hint` : null, error ? `${id}-error` : null]
      .filter((value): value is string => Boolean(value))
      .join(' ');

    if (describedBy) {
      control.setAttribute('aria-describedby', describedBy);
    } else {
      control.removeAttribute('aria-describedby');
    }
  });
</script>

<div class="form-field" bind:this={fieldElement}>
  <label class="field-label" for={id}>
    {label}
    {#if required}
      <span class="required-mark" aria-hidden="true">*</span>
    {/if}
  </label>
  {@render children()}
  {#if hint}
    <p id={`${id}-hint`} class="field-hint">{hint}</p>
  {/if}
  {#if error}
    <p id={`${id}-error`} class="field-error" role="alert">{error}</p>
  {/if}
</div>

<style>
  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-label {
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--color-ink);
  }

  .required-mark {
    margin-left: 0.15rem;
    color: var(--color-danger);
  }

  .field-hint {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .field-error {
    color: var(--color-danger);
    font-size: var(--text-xs);
    font-weight: 650;
  }
</style>
