<script lang="ts">
  import type { Snippet } from 'svelte';
  import { onMount } from 'svelte';

  import '#lib/styles/tokens.css';
  import { call, isTauriRuntime } from '#lib/api/invoke.js';
  import AppShell from '#lib/components/AppShell.svelte';
  import { navigationService } from '#lib/services/navigationService.svelte.js';
  import { themeService } from '#lib/services/themeService.svelte.js';

  let { children }: { children: Snippet } = $props();

  let startupNotice = $state<string | null>(null);

  navigationService.init();

  function preventContextMenu(event: MouseEvent): void {
    event.preventDefault();
  }

  onMount(() => {
    themeService.init();

    if (!isTauriRuntime()) {
      return;
    }

    void call<string | null>('take_startup_notice')
      .then((notice) => {
        startupNotice = notice;
      })
      .catch((cause) => {
        console.error('No se pudo leer el aviso de inicio.', cause);
      });
  });
</script>

<svelte:window oncontextmenu={preventContextMenu} />

<svelte:head>
  <title>Gestión de Limpieza</title>
</svelte:head>

<AppShell notice={startupNotice}>
  {@render children()}
</AppShell>
