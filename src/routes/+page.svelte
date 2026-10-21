<script lang="ts">
  import EmptyState from '#lib/components/EmptyState.svelte';
  import { formatLongDate } from '#lib/utils/format.js';

  const todayLabel = formatLongDate(new Date());
</script>

<svelte:head>
  <title>Tablero · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">{todayLabel}</p>
    <h1 class="page-title">Tablero del día</h1>
    <p class="page-lede">Qué hay que limpiar hoy y quién lo hace.</p>
  </div>
</header>

<section class="board panel" aria-label="Tablero sin servicios">
  <EmptyState
    title="Todavía no hay nada para mostrar"
    description="Cuando cargues al personal y los lugares, acá vas a ver los servicios del día, las tareas sugeridas y las incidencias abiertas."
  />
</section>

<style>
  .board {
    display: grid;
    min-height: 27rem;
    place-items: center;
    background:
      linear-gradient(135deg, var(--color-surface-raised), var(--color-surface) 58%),
      var(--color-surface);
  }
</style>
