<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { asset } from '$app/paths';

  import { isTauriRuntime } from '#lib/api/invoke.js';
  import type { Gender } from '#lib/api/types.js';

  interface Props {
    photoUrl?: string | null;
    gender: Gender;
    alt: string;
    size?: 'sm' | 'md' | 'lg';
  }

  let { photoUrl = null, gender, alt, size = 'md' }: Props = $props();

  let failedUrl = $state<string | null>(null);

  const fallbackSrc = $derived(
    gender === 'female' ? asset('avatars/default-female.svg') : asset('avatars/default-male.svg')
  );

  const resolvedSrc = $derived.by(() => {
    if (!photoUrl || photoUrl === failedUrl) {
      return fallbackSrc;
    }
    if (
      photoUrl.startsWith('http://') ||
      photoUrl.startsWith('https://') ||
      photoUrl.startsWith('asset:') ||
      photoUrl.startsWith('data:') ||
      photoUrl.startsWith('blob:')
    ) {
      return photoUrl;
    }
    if (isTauriRuntime()) {
      return convertFileSrc(photoUrl);
    }
    return fallbackSrc;
  });

  function handleError(): void {
    if (photoUrl) {
      failedUrl = photoUrl;
    }
  }
</script>

<img
  class={`avatar avatar-${size}`}
  src={resolvedSrc}
  {alt}
  loading="lazy"
  decoding="async"
  onerror={handleError}
/>

<style>
  .avatar {
    display: block;
    flex: none;
    border-radius: 50%;
    border: 1px solid var(--color-line-strong);
    background: var(--color-surface-muted);
    object-fit: cover;
  }

  .avatar-sm {
    width: 2.25rem;
    height: 2.25rem;
  }

  .avatar-md {
    width: 3rem;
    height: 3rem;
  }

  .avatar-lg {
    width: 4.5rem;
    height: 4.5rem;
  }
</style>
