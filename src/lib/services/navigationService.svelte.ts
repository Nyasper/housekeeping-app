import { afterNavigate } from '$app/navigation';

/**
 * Tracks the app's in-memory navigation history so the sidebar can offer
 * back/forward controls. SvelteKit drives the actual browser history; this
 * service only mirrors it to know when each direction is available.
 */
export class NavigationService {
  canGoBack = $state(false);
  canGoForward = $state(false);

  #history: string[] = [];
  #index = -1;
  #initialized = false;

  /** Must be called during component initialization (root layout). */
  init(): void {
    if (this.#initialized) {
      return;
    }
    this.#initialized = true;

    afterNavigate((navigation) => {
      const href = navigation.to?.url.href ?? window.location.href;

      if (navigation.type === 'enter') {
        this.#history = [href];
        this.#index = 0;
      } else if (navigation.type === 'popstate') {
        const match = this.#find(href);
        if (match !== -1) {
          this.#index = match;
        } else {
          this.#push(href);
        }
      } else {
        this.#push(href);
      }

      this.#syncAvailability();
    });
  }

  goBack(): void {
    if (this.canGoBack) {
      window.history.back();
    }
  }

  goForward(): void {
    if (this.canGoForward) {
      window.history.forward();
    }
  }

  #push(href: string): void {
    if (this.#history[this.#index] === href) {
      return;
    }
    this.#history = [...this.#history.slice(0, this.#index + 1), href];
    this.#index = this.#history.length - 1;
  }

  #find(href: string): number {
    for (let index = this.#index - 1; index >= 0; index -= 1) {
      if (this.#history[index] === href) {
        return index;
      }
    }
    for (let index = this.#index + 1; index < this.#history.length; index += 1) {
      if (this.#history[index] === href) {
        return index;
      }
    }
    return -1;
  }

  #syncAvailability(): void {
    this.canGoBack = this.#index > 0;
    this.canGoForward = this.#index >= 0 && this.#index < this.#history.length - 1;
  }
}

export const navigationService = new NavigationService();
