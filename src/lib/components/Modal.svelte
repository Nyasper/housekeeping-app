<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    title: string;
    description?: string;
    dirty?: boolean;
    busy?: boolean;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    formId?: string;
    submitLabel?: string;
    submitDisabled?: boolean;
    cancelLabel?: string;
    onclose: () => void;
    footer?: Snippet;
    children: Snippet;
  }

  let {
    open,
    title,
    description = '',
    dirty = false,
    busy = false,
    size = 'md',
    formId = '',
    submitLabel = 'Guardar',
    submitDisabled = false,
    cancelLabel = 'Cancelar',
    onclose,
    footer,
    children
  }: Props = $props();

  const uid = $props.id();
  const titleId = `modal-title-${uid}`;
  const descriptionId = `modal-description-${uid}`;

  function syncDialog(dialog: HTMLDialogElement) {
    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }

  function closeExplicitly(): void {
    if (!busy) {
      onclose();
    }
  }

  function handleCancel(event: Event): void {
    event.preventDefault();
    if (!busy && !dirty) {
      onclose();
    }
  }

  function handleBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget && !busy && !dirty) {
      onclose();
    }
  }
</script>

<dialog
  {@attach syncDialog}
  class="modal panel size-{size}"
  aria-labelledby={titleId}
  aria-describedby={description ? descriptionId : undefined}
  oncancel={handleCancel}
  onclick={handleBackdropClick}
>
  <header class="modal-header">
    <div class="modal-heading">
      <h2 id={titleId}>{title}</h2>
      {#if description}
        <p id={descriptionId}>{description}</p>
      {/if}
    </div>
    <button
      type="button"
      class="modal-close"
      aria-label="Cerrar"
      title="Cerrar"
      onclick={closeExplicitly}
      disabled={busy}
    >
      <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="m6 6 12 12M18 6 6 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      </svg>
    </button>
  </header>

  <div class="modal-body">
    {@render children()}
  </div>

  {#if footer || formId}
    <footer class="modal-footer">
      {#if formId}
        <button
          type="submit"
          form={formId}
          class="button button-primary"
          disabled={busy || submitDisabled}
        >
          {busy ? 'Guardando…' : submitLabel}
        </button>
      {/if}
      {@render footer?.()}
      {#if formId}
        <button
          type="button"
          class="button button-secondary"
          onclick={closeExplicitly}
          disabled={busy}
        >
          {cancelLabel}
        </button>
      {/if}
    </footer>
  {/if}
</dialog>

<style>
  .modal {
    width: min(40rem, calc(100vw - 2rem));
    max-height: min(90vh, 54rem);
    padding: 0;
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-xl);
    background: var(--color-surface);
    color: var(--color-ink);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .modal[open] {
    display: flex;
    flex-direction: column;
  }

  .modal.size-sm {
    width: min(30rem, calc(100vw - 2rem));
  }

  .modal.size-lg {
    width: min(52rem, calc(100vw - 2rem));
  }

  .modal.size-xl {
    width: min(64rem, calc(100vw - 2rem));
  }

  .modal::backdrop {
    background: rgb(10 18 16 / 0.48);
    backdrop-filter: blur(2px);
  }

  .modal-header {
    display: flex;
    flex: none;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-6);
    border-bottom: 1px solid var(--color-line);
  }

  .modal-heading {
    min-width: 0;
  }

  .modal-heading h2 {
    font-size: var(--text-xl);
  }

  .modal-heading p {
    margin-top: var(--space-1);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .modal-close {
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: 2.25rem;
    height: 2.25rem;
    margin: -0.25rem -0.5rem 0 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-ink-muted);
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
  }

  .modal-close:hover:not(:disabled) {
    background: var(--color-surface-muted);
    color: var(--color-ink);
  }

  .modal-close:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .modal-close svg {
    width: 1.1rem;
    height: 1.1rem;
  }

  .modal-body {
    flex: 1;
    min-height: 0;
    padding: var(--space-5) var(--space-6);
    overflow-y: auto;
  }

  .modal-footer {
    display: flex;
    flex: none;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-6);
    border-top: 1px solid var(--color-line);
    background: var(--color-surface-raised);
  }

  @media (max-width: 640px) {
    .modal-header,
    .modal-body,
    .modal-footer {
      padding-inline: var(--space-4);
    }
  }
</style>
