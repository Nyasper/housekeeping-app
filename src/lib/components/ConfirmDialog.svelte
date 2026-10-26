<script lang="ts">
  interface Props {
    open: boolean;
    title: string;
    description: string;
    confirmLabel?: string;
    cancelLabel?: string;
    tone?: 'primary' | 'danger';
    busy?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let {
    open,
    title,
    description,
    confirmLabel = 'Confirmar',
    cancelLabel = 'Cancelar',
    tone = 'primary',
    busy = false,
    onconfirm,
    oncancel
  }: Props = $props();

  const uid = $props.id();
  const titleId = `confirm-dialog-title-${uid}`;
  const descriptionId = `confirm-dialog-desc-${uid}`;

  function syncDialog(dialog: HTMLDialogElement) {
    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }

  function handleDialogCancel(event: Event): void {
    event.preventDefault();
    if (!busy) {
      oncancel();
    }
  }

  function handleBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget && !busy) {
      oncancel();
    }
  }
</script>

<dialog
  {@attach syncDialog}
  class="confirm-dialog panel"
  aria-labelledby={titleId}
  aria-describedby={descriptionId}
  oncancel={handleDialogCancel}
  onclick={handleBackdropClick}
>
  <div class="dialog-body">
    <h2 id={titleId}>{title}</h2>
    <p id={descriptionId}>{description}</p>
    <div class="dialog-actions">
      <button type="button" class="button button-secondary" onclick={oncancel} disabled={busy}>
        {cancelLabel}
      </button>
      <button
        type="button"
        class={`button ${tone === 'danger' ? 'button-danger' : 'button-primary'}`}
        onclick={onconfirm}
        disabled={busy}
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</dialog>

<style>
  .confirm-dialog {
    width: min(30rem, calc(100vw - 2rem));
    padding: 0;
    border: 1px solid var(--color-line-strong);
    border-radius: var(--radius-xl);
    background: var(--color-surface);
    color: var(--color-ink);
    box-shadow: var(--shadow-lg);
  }

  .confirm-dialog::backdrop {
    background: rgb(10 18 16 / 0.48);
    backdrop-filter: blur(2px);
  }

  .dialog-body {
    padding: var(--space-6);
  }

  h2 {
    font-size: var(--text-xl);
  }

  p {
    margin-top: var(--space-3);
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    margin-top: var(--space-5);
  }

  .button-danger {
    background: var(--color-danger);
    color: var(--color-on-danger);
  }

  .button-danger:hover:not(:disabled) {
    filter: brightness(0.92);
  }
</style>
