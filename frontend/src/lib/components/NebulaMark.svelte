<script lang="ts">
  import { cn } from '$lib/utils.js'

  // The Nebula mark. Decorative: the text next to it or the page heading
  // carries the meaning, so it is hidden from assistive technology.
  let { class: className = '' }: { class?: string } = $props()
</script>

<img
  src="/favicon-32x32.png"
  alt=""
  width="32"
  height="32"
  class={cn('size-6', className)}
/>
