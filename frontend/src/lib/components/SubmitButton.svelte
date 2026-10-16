<script lang="ts">
  import type { Snippet } from 'svelte'
  import Loader2Icon from '@lucide/svelte/icons/loader-2'
  import { Button, type ButtonVariant } from '$lib/components/ui/button/index.js'

  // A form's submit button: disabled with a spinner while the Inertia form
  // is processing, so a second click cannot submit twice.
  let {
    processing = false,
    variant = 'default',
    form,
    class: className = '',
    children,
  }: {
    processing?: boolean
    variant?: ButtonVariant
    form?: string
    class?: string
    children: Snippet
  } = $props()
</script>

<Button
  type="submit"
  {variant}
  {form}
  class={className}
  disabled={processing}
  aria-busy={processing}
>
  {#if processing}
    <Loader2Icon class="animate-spin" data-icon="inline-start" />
  {/if}
  {@render children()}
</Button>
