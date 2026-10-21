import { errorMessage } from '#lib/api/errors.js';
import { call } from '#lib/api/invoke.js';
import type {
  Place,
  PlaceInput,
  PlaceStatus,
  RecommendedTask
} from '#lib/api/types.js';
import { todayIso } from '#lib/utils/format.js';

export class PlacesService {
  items = $state<Place[]>([]);
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  referenceDate = $state<string | null>(null);

  async load(date?: string): Promise<void> {
    if (date !== undefined) {
      this.referenceDate = date;
    }
    this.loading = true;
    this.error = null;
    try {
      this.items = await call<Place[]>('places_list', {
        date: this.referenceDate ?? null
      });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async create(input: PlaceInput): Promise<Place | null> {
    this.saving = true;
    this.error = null;
    try {
      const created = await call<Place>('places_create', { input });
      await this.load();
      return created;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async update(id: number, input: PlaceInput): Promise<Place | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Place>('places_update', { id, input });
      await this.load();
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async setStatusOverride(place: Place, status: PlaceStatus | null): Promise<Place | null> {
    return this.update(place.id, {
      name: place.name,
      type: place.type,
      status,
      location: place.location,
      check_in_date: place.check_in_date,
      check_out_date: place.check_out_date,
      notes: place.notes
    });
  }

  async deactivate(id: number): Promise<Place | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Place>('places_deactivate', { id });
      await this.load();
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async reactivate(id: number): Promise<Place | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Place>('places_reactivate', { id });
      await this.load();
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async recommendedTask(placeId: number, date: string = todayIso()): Promise<RecommendedTask | null> {
    try {
      return await call<RecommendedTask>('places_recommended_task', { placeId, date });
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    }
  }
}

export const placesService = new PlacesService();
