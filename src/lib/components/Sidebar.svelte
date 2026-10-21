<script lang="ts">
  import { page } from '$app/state';

  import ThemeToggle from './ThemeToggle.svelte';

  const links = [
    { href: '/', label: 'Tablero', icon: 'board' },
    { href: '/staff', label: 'Personal', icon: 'staff' },
    { href: '/places', label: 'Lugares', icon: 'places' },
    { href: '/cleaning', label: 'Servicios', icon: 'cleaning' },
    { href: '/settings', label: 'Ajustes', icon: 'settings' }
  ] as const;

  const pathname = $derived(page.url.pathname);

  function isActive(href: string): boolean {
    return href === '/' ? pathname === '/' : pathname.startsWith(href);
  }
</script>

<aside class="sidebar">
  <a class="brand" href="/" aria-label="Limpieza, ir al tablero">
    <span class="brand-mark" aria-hidden="true">
      <span></span>
      <span></span>
      <span></span>
    </span>
    <span class="brand-text">
      <span class="brand-name">Limpieza</span>
      <span class="brand-tagline">Gestión del edificio</span>
    </span>
  </a>

  <nav aria-label="Navegación principal">
    <ul class="nav-list">
      {#each links as link (link.href)}
        <li>
          <a
            href={link.href}
            class={`nav-link ${isActive(link.href) ? 'is-active' : ''}`}
            aria-current={isActive(link.href) ? 'page' : undefined}
          >
            <span class="nav-icon" aria-hidden="true">
              {#if link.icon === 'board'}
                <svg viewBox="0 0 24 24" fill="none">
                  <rect x="4" y="4" width="6.5" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.7" />
                  <rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.7" />
                  <rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.7" />
                  <rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.7" />
                </svg>
              {:else if link.icon === 'staff'}
                <svg viewBox="0 0 24 24" fill="none">
                  <circle cx="9.5" cy="8" r="3.2" stroke="currentColor" stroke-width="1.7" />
                  <path
                    d="M4 19c1.2-3.4 3.5-5 5.5-5s4.3 1.6 5.5 5M15.5 6.5a2.8 2.8 0 0 1 0 5.2M17 14.5c1.5.7 2.6 2 3.2 4.5"
                    stroke="currentColor"
                    stroke-width="1.7"
                    stroke-linecap="round"
                  />
                </svg>
              {:else if link.icon === 'places'}
                <svg viewBox="0 0 24 24" fill="none">
                  <path
                    d="M5 20V5.5A1.5 1.5 0 0 1 6.5 4h11A1.5 1.5 0 0 1 19 5.5V20M3 20h18M9 8h2m2 0h2M9 12h2m2 0h2M10 20v-4h4v4"
                    stroke="currentColor"
                    stroke-width="1.7"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              {:else if link.icon === 'cleaning'}
                <svg viewBox="0 0 24 24" fill="none">
                  <path
                    d="M9 5h6m-7 3h8a2 2 0 0 1 2 2v8.5a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V10a2 2 0 0 1 2-2Zm1.5 6.2 1.8 1.8 3.4-3.6"
                    stroke="currentColor"
                    stroke-width="1.7"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="none">
                  <path
                    d="M4 7h9m3 0h4M4 17h4m3 0h9"
                    stroke="currentColor"
                    stroke-width="1.7"
                    stroke-linecap="round"
                  />
                  <circle cx="15" cy="7" r="2.2" stroke="currentColor" stroke-width="1.7" />
                  <circle cx="9" cy="17" r="2.2" stroke="currentColor" stroke-width="1.7" />
                </svg>
              {/if}
            </span>
            <span>{link.label}</span>
          </a>
        </li>
      {/each}
    </ul>
  </nav>

  <div class="sidebar-footer">
    <ThemeToggle />
    <p class="offline-note">
      <span class="status-dot" aria-hidden="true"></span>
      Todos los datos quedan en este equipo.
    </p>
  </div>
</aside>

<style>
  .sidebar {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100vh;
    overflow-y: auto;
    padding: var(--space-5) var(--space-4);
    border-right: 1px solid var(--color-line);
    background: var(--color-sidebar);
  }

  .brand {
    display: flex;
    min-height: 2.75rem;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2);
    border-radius: var(--radius-md);
  }

  .brand:hover {
    background: var(--color-surface-muted);
  }

  .brand-mark {
    display: grid;
    flex: none;
    width: 2.35rem;
    height: 2.35rem;
    align-content: center;
    gap: 3px;
    padding: 0.55rem 0.5rem;
    border-radius: 0.7rem;
    background: var(--color-primary);
  }

  .brand-mark span {
    display: block;
    height: 2px;
    border-radius: 2px;
    background: var(--color-on-primary);
  }

  .brand-mark span:nth-child(1) {
    width: 100%;
  }

  .brand-mark span:nth-child(2) {
    width: 70%;
  }

  .brand-mark span:nth-child(3) {
    width: 88%;
  }

  .brand-text {
    display: flex;
    min-width: 0;
    flex-direction: column;
  }

  .brand-name {
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 650;
    letter-spacing: -0.01em;
    line-height: 1.25;
  }

  .brand-tagline {
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    line-height: 1.35;
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 2.75rem;
    padding: 0.55rem 0.7rem;
    border-radius: var(--radius-md);
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    font-weight: 600;
    transition:
      background-color var(--transition-fast),
      color var(--transition-fast);
  }

  .nav-link:hover {
    background: var(--color-surface-muted);
    color: var(--color-ink);
  }

  .nav-link.is-active {
    background: var(--color-primary-soft);
    color: var(--color-primary-strong);
  }

  .nav-icon {
    display: inline-flex;
    flex: none;
    width: 1.2rem;
    height: 1.2rem;
  }

  .nav-icon svg {
    width: 100%;
    height: 100%;
  }

  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: auto;
  }

  .offline-note {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-2);
    color: var(--color-ink-muted);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .status-dot {
    flex: none;
    width: 0.4rem;
    height: 0.4rem;
    border-radius: 50%;
    background: var(--color-success);
  }

  @media (max-width: 900px) {
    .sidebar {
      position: sticky;
      top: 0;
      z-index: 20;
      display: flex;
      height: auto;
      flex-flow: row wrap;
      align-items: center;
      gap: var(--space-3);
      padding: var(--space-3) var(--space-4);
      border-right: 0;
      border-bottom: 1px solid var(--color-line);
    }

    .brand {
      order: 1;
      flex: 1;
      padding: var(--space-1);
    }

    .brand-tagline {
      display: none;
    }

    nav {
      order: 3;
      width: 100%;
    }

    .nav-list {
      flex-direction: row;
    }

    .nav-list li {
      flex: 1;
    }

    .nav-link {
      justify-content: center;
    }

    .sidebar-footer {
      order: 2;
      margin-top: 0;
      margin-left: auto;
    }

    .offline-note {
      display: none;
    }
  }
</style>
