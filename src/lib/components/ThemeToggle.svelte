<script lang="ts">
  import { themeService } from '#lib/services/themeService.svelte.js';

  const isDark = $derived(themeService.theme === 'dark');
  const actionLabel = $derived(isDark ? 'Cambiar a tema claro' : 'Cambiar a tema oscuro');
</script>

<button
  type="button"
  class="theme-toggle"
  onclick={() => themeService.toggle()}
  aria-label={actionLabel}
  title={actionLabel}
>
  <span class="theme-icon" aria-hidden="true">
    {#if isDark}
      <svg viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="4.2" stroke="currentColor" stroke-width="1.7" />
        <path
          d="M12 2.8v2M12 19.2v2M4.6 4.6l1.4 1.4m12 12 1.4 1.4M2.8 12h2m14.4 0h2M4.6 19.4 6 18m12-12 1.4-1.4"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
        />
      </svg>
    {:else}
      <svg viewBox="0 0 24 24" fill="none">
        <path
          d="M20.2 14.2A8.4 8.4 0 0 1 9.8 3.8 8.5 8.5 0 1 0 20.2 14.2Z"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    {/if}
  </span>
  <span class="theme-label">{actionLabel}</span>
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    width: 100%;
    min-height: 2.75rem;
    align-items: center;
    gap: var(--space-3);
    padding: 0.55rem 0.7rem;
    border: 1px solid var(--color-line);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    font-weight: 600;
    transition:
      background-color var(--transition-fast),
      border-color var(--transition-fast),
      color var(--transition-fast);
  }

  .theme-toggle:hover {
    border-color: var(--color-line-strong);
    background: var(--color-surface-muted);
    color: var(--color-ink);
  }

  .theme-icon {
    display: inline-flex;
    flex: none;
    width: 1.2rem;
    height: 1.2rem;
  }

  .theme-icon svg {
    width: 100%;
    height: 100%;
  }

  @media (max-width: 900px) {
    .theme-toggle {
      width: auto;
      padding: 0.55rem;
    }

    .theme-label {
      display: none;
    }
  }
</style>
