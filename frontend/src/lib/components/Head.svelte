<script lang="ts">
  import type { Snippet } from 'svelte'
  import { pageTitle } from '$lib/title.js'

  // A page's tab title, through Svelte's own `<svelte:head>`: Inertia's
  // Svelte adapter has no Head component. Server rendering puts the title
  // in the document head and the server then leaves out its default one.
  let { title = '', children }: { title?: string; children?: Snippet } = $props()
</script>

<svelte:head>
  <title>{pageTitle(title)}</title>
  {@render children?.()}
</svelte:head>
