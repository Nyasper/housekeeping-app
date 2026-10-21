<script lang="ts">
  import type { Snippet } from 'svelte';

  import Sidebar from './Sidebar.svelte';

  interface Props {
    children: Snippet;
    notice?: string | null;
  }

  let { children, notice = null }: Props = $props();

  let noticeDismissed = $state(false);
</script>

<div class="app-shell">
  <a class="skip-link" href="#contenido">Saltar al contenido</a>

  <Sidebar />

  <div class="workspace">
    {#if notice && !noticeDismissed}
      <div class="notice" role="status">
        <svg class="notice-icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M12 8v5m0 3.5v.5M10.3 4.6 3.5 16.4A2 2 0 0 0 5.2 19.4h13.6a2 2 0 0 0 1.7-3L13.7 4.6a2 2 0 0 0-3.4 0Z"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <p>{notice}</p>
        <button
          type="button"
          class="notice-close"
          aria-label="Cerrar aviso"
          onclick={() => (noticeDismissed = true)}
        >
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="m6 6 12 12M18 6 6 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    {/if}

    <main id="contenido" class="content" tabindex="-1">
      {@render children()}
    </main>
  </div>
</div>

<style>
  .app-shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
    min-height: 100vh;
  }

  .skip-link {
    position: fixed;
    top: var(--space-3);
    left: var(--space-3);
    z-index: 100;
    padding: 0.6rem 1rem;
    border-radius: var(--radius-md);
    background: var(--color-primary);
    color: var(--color-on-primary);
    font-size: var(--text-sm);
    font-weight: 650;
    transform: translateY(-180%);
    transition: transform var(--transition-fast);
  }

  .skip-link:focus-visible {
    transform: translateY(0);
  }

  .workspace {
    position: relative;
    min-width: 0;
    padding: var(--space-7) var(--space-6) var(--space-8);
  }

  .workspace::before {
    content: '';
    position: fixed;
    inset: 0 0 0 var(--sidebar-width);
    background-image:
      linear-gradient(var(--grid-line) 1px, transparent 1px),
      linear-gradient(90deg, var(--grid-line) 1px, transparent 1px);
    background-size: 5.5rem 5.5rem;
    mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 0.7), transparent 62%);
    pointer-events: none;
  }

  .content {
    position: relative;
    z-index: 1;
    max-width: var(--content-width);
    margin-inline: auto;
  }

  .notice {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    max-width: var(--content-width);
    margin: 0 auto var(--space-5);
    padding: var(--space-4);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-lg);
    background: var(--color-warning-soft);
    color: var(--color-ink);
  }

  .notice p {
    flex: 1;
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .notice-icon {
    flex: none;
    width: 1.25rem;
    height: 1.25rem;
    margin-top: 0.1rem;
    color: var(--color-warning);
  }

  .notice-close {
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    margin: -0.35rem -0.35rem 0 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-ink-muted);
  }

  .notice-close:hover {
    background: rgb(0 0 0 / 0.06);
    color: var(--color-ink);
  }

  .notice-close svg {
    width: 1rem;
    height: 1rem;
  }

  @media (max-width: 900px) {
    .app-shell {
      grid-template-columns: minmax(0, 1fr);
    }

    .workspace {
      padding: var(--space-5) var(--space-4) var(--space-7);
    }

    .workspace::before {
      inset: 0;
    }

    .content {
      scroll-margin-top: var(--space-5);
    }
  }
</style>
