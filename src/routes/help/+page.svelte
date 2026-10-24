<svelte:head>
  <title>Ayuda · Gestión de Limpieza</title>
</svelte:head>

<header class="page-header">
  <div>
    <p class="date-line">Guía de uso</p>
    <h1 class="page-title">Ayuda</h1>
    <p class="page-lede">
      Todo lo necesario para organizar la limpieza del edificio: personal, lugares, servicios,
      incidencias, reportes y respaldos.
    </p>
  </div>
  <a class="button button-secondary" href="/">Ir al tablero</a>
</header>

<section class="panel intro" aria-labelledby="help-start">
  <h2 id="help-start">Cómo empezar</h2>
  <ol>
    <li>Registrá primero los <strong>lugares</strong>: apartamentos y áreas comunes.</li>
    <li>Cargá al <strong>personal</strong>, sus días de descanso y sus apartamentos principales.</li>
    <li>Creá los <strong>servicios de limpieza</strong> del día y asigná quién los realiza.</li>
    <li>Usá el <strong>tablero diario</strong> para seguir lo pendiente y registrar incidencias.</li>
  </ol>
</section>

<div class="help-grid">
  <section class="panel help-card" aria-labelledby="help-board">
    <h2 id="help-board">Tablero diario</h2>
    <ul>
      <li>Muestra la fecha activa y permite moverte al día anterior, al siguiente o volver a hoy.</li>
      <li>Reúne las tareas recomendadas por apartamento, los servicios del día, el personal disponible y las incidencias abiertas.</li>
      <li>Desde cada apartamento podés crear un servicio con el lugar y la tarea ya sugeridos.</li>
      <li>Los servicios se pueden iniciar, completar o cancelar según su estado.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-staff">
    <h2 id="help-staff">Personal</h2>
    <ul>
      <li>Datos personales, cédula única, rol, género y foto opcional con avatar predeterminado.</li>
      <li>Modalidad zafral con fechas de temporada y días de descanso semanal.</li>
      <li>Ausencias por certificado médico, vacaciones o falta puntual, sin superposiciones.</li>
      <li>Los funcionarios se dan de baja sin perder su historial y se pueden reactivar.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-places">
    <h2 id="help-places">Lugares</h2>
    <ul>
      <li>Apartamentos y áreas comunes con nombre único y ubicación opcional.</li>
      <li>Las fechas de ingreso y egreso permiten calcular el estado de ocupación.</li>
      <li>La app sugiere la tarea de limpieza según la estadía: post check-out, pre check-in, limpieza diaria o retoque.</li>
      <li>Podés omitir el turno de un apartamento o desactivar un lugar sin borrar su historial.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-cleaning">
    <h2 id="help-cleaning">Servicios de limpieza</h2>
    <ul>
      <li>Cada servicio tiene lugar, fecha, tipo, estado, horarios y al menos un funcionario asignado.</li>
      <li>Las sugerencias ordenan al personal según disponibilidad, apartamento principal, rol y carga del día.</li>
      <li>Solo puede haber un responsable principal por servicio.</li>
      <li>Al completar, indicá hora de inicio y fin para calcular la duración real.</li>
      <li>Un servicio completado o cancelado se puede reabrir con confirmación explícita.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-incidents">
    <h2 id="help-incidents">Incidencias</h2>
    <ul>
      <li>Registrá daños, objetos olvidados o faltantes de insumos asociados a un servicio.</li>
      <li>Podés adjuntar una foto y ampliarla desde la tarjeta de la incidencia.</li>
      <li>Filtrá por estado, tipo, fecha o servicio.</li>
      <li>Marcá una incidencia como resuelta o reabrirla si vuelve a ser necesaria.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-reports">
    <h2 id="help-reports">Reportes y exportaciones</h2>
    <ul>
      <li>Servicios y horas por funcionario, duración promedio por tipo y ausencias por mes.</li>
      <li>Planilla diaria con los servicios y totales del día seleccionado.</li>
      <li>Exportá el reporte visible a CSV, XLSX o PDF desde la barra de período.</li>
      <li>Los archivos exportados usan siempre un formato claro y legible, sin importar el tema activo.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-settings">
    <h2 id="help-settings">Ajustes y respaldos</h2>
    <ul>
      <li>Configurá los días de ventana para recomendaciones de check-out y check-in.</li>
      <li>Creá un respaldo ZIP con la base de datos, las fotos y un manifiesto.</li>
      <li>Restaurá un respaldo verificado; la app conserva una copia de seguridad previa.</li>
      <li>Guardá los respaldos fuera del equipo para proteger la información.</li>
    </ul>
  </section>

  <section class="panel help-card" aria-labelledby="help-windows">
    <h2 id="help-windows">Ventanas y atajos</h2>
    <ul>
      <li>Las acciones de creación y edición se abren en ventanas modales.</li>
      <li>Hacé clic fuera de una ventana o presioná <kbd>Esc</kbd> para cerrarla si no hiciste cambios.</li>
      <li>Si ya hay cambios sin guardar, usá la <strong>X</strong> o <strong>Cancelar</strong> para descartarlos.</li>
      <li><kbd>Enter</kbd> envía el formulario; <kbd>Tab</kbd> recorre los campos y acciones.</li>
      <li>Las flechas del panel lateral permiten volver atrás o adelante en la navegación.</li>
    </ul>
  </section>
</div>

<section class="panel local-note" aria-labelledby="help-data">
  <h2 id="help-data">Datos y privacidad</h2>
  <p>
    La aplicación funciona de forma local en este equipo. La base de datos y las fotos se guardan
    en la carpeta de datos de la aplicación; no se envían a ningún servicio externo.
  </p>
</section>

<style>
  .intro {
    padding: var(--space-5) var(--space-6);
    margin-bottom: var(--space-5);
  }

  .intro h2 {
    font-size: var(--text-lg);
    margin-bottom: var(--space-3);
  }

  .intro ol {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding-left: 1.25rem;
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }

  .help-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr));
    gap: var(--space-4);
    margin-bottom: var(--space-5);
  }

  .help-card {
    padding: var(--space-5);
  }

  .help-card h2 {
    font-size: var(--text-lg);
    margin-bottom: var(--space-3);
  }

  .help-card ul {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding-left: 1.1rem;
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  kbd {
    padding: 0.1rem 0.4rem;
    border: 1px solid var(--color-line-strong);
    border-bottom-width: 2px;
    border-radius: var(--radius-sm);
    background: var(--color-surface-raised);
    color: var(--color-ink);
    font-family: var(--font-mono);
    font-size: 0.75em;
  }

  .local-note {
    padding: var(--space-5) var(--space-6);
    border-color: var(--color-primary);
    background: var(--color-primary-soft);
  }

  .local-note h2 {
    font-size: var(--text-lg);
  }

  .local-note p {
    margin-top: var(--space-2);
    max-width: 70ch;
    color: var(--color-ink-soft);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }

  @media (max-width: 640px) {
    .intro,
    .local-note {
      padding: var(--space-4);
    }
  }
</style>
