import { errorMessage } from '#lib/api/errors.js';
import { call } from '#lib/api/invoke.js';
import type {
  Incident,
  IncidentInput,
  IncidentListFilters,
  IncidentStatus
} from '#lib/api/types.js';

export class IncidentsService {
  items = $state<Incident[]>([]);
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  lastFilters = $state<IncidentListFilters>({});

  readonly openItems = $derived(this.items.filter((incident) => incident.status === 'open'));

  async load(filters?: IncidentListFilters): Promise<void> {
    if (filters !== undefined) {
      this.lastFilters = filters;
    }
    this.loading = true;
    this.error = null;
    try {
      this.items = await call<Incident[]>('incidents_list', {
        filters: this.lastFilters
      });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async create(input: IncidentInput): Promise<Incident | null> {
    this.saving = true;
    this.error = null;
    try {
      const created = await call<Incident>('incidents_create', { input });
      await this.load();
      return created;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async setStatus(id: number, status: IncidentStatus): Promise<Incident | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Incident>('incidents_set_status', { id, status });
      await this.load();
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }
}

export const incidentsService = new IncidentsService();
