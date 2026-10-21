<script lang="ts">
  interface Props {
    message: string | null;
    tone?: 'success' | 'warning' | 'error';
    ondismiss?: () => void;
  }

  let { message, tone = 'success', ondismiss }: Props = $props();
</script>

{#if message}
  <div
    class={`toast toast-${tone}`}
    role={tone === 'error' ? 'alert' : 'status'}
    aria-live={tone === 'error' ? 'assertive' : 'polite'}
  >
    <p>{message}</p>
    {#if ondismiss}
      <button type="button" class="toast-close" aria-label="Cerrar notificación" onclick={ondismiss}>
        <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="m6 6 12 12M18 6 6 18"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
          />
        </svg>
      </button>
    {/if}
  </div>
{/if}

<style>
  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-5);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    font-size: var(--text-sm);
    font-weight: 550;
    box-shadow: var(--shadow-sm);
  }

  .toast p {
    flex: 1;
    line-height: var(--leading-snug);
  }

  .toast-success {
    border: 1px solid var(--color-success);
    background: var(--color-success-soft);
    color: var(--color-ink);
  }

  .toast-warning {
    border: 1px solid var(--color-warning);
    background: var(--color-warning-soft);
    color: var(--color-ink);
  }

  .toast-error {
    border: 1px solid var(--color-danger);
    background: var(--color-danger-soft);
    color: var(--color-ink);
  }

  .toast-close {
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-ink-soft);
  }

  .toast-close:hover {
    background: rgb(0 0 0 / 0.06);
    color: var(--color-ink);
  }

  .toast-close svg {
    width: 1rem;
    height: 1rem;
  }
</style>
