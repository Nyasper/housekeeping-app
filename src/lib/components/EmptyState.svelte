<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    description: string;
    children?: Snippet;
  }

  let { title, description, children }: Props = $props();
</script>

<div class="empty-state">
  <div class="illustration" aria-hidden="true">
    <svg viewBox="0 0 120 120" fill="none">
      <rect x="14" y="18" width="92" height="84" rx="10" stroke="currentColor" stroke-width="2" />
      <path d="M56 18v50M14 68h92M72 68v34" stroke="currentColor" stroke-width="2" />
      <path d="M30 34h12M30 44h12M84 82h8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      <circle cx="41" cy="86" r="7" fill="currentColor" opacity="0.16" />
      <path
        d="M38.5 86.2 40.6 88l3.2-4"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </div>
  <h2>{title}</h2>
  <p>{description}</p>
  {#if children}
    <div class="empty-actions">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    max-width: 34rem;
    flex-direction: column;
    align-items: center;
    margin-inline: auto;
    padding: var(--space-7) var(--space-5);
    text-align: center;
  }

  .illustration {
    width: 6.5rem;
    margin-bottom: var(--space-4);
    color: var(--color-primary);
    opacity: 0.85;
  }

  .illustration svg {
    display: block;
    width: 100%;
    height: auto;
  }

  h2 {
    font-size: var(--text-xl);
    letter-spacing: -0.01em;
  }

  p {
    margin-top: var(--space-2);
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
    line-height: var(--leading-snug);
  }

  .empty-actions {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-5);
  }
</style>
