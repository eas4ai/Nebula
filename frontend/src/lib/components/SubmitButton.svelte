<script lang="ts">
  import { onMount, type Snippet } from 'svelte'
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

  // The button is also disabled until the page's code has taken over. The
  // server renders the form before the browser runs any of it, and a click
  // or Enter in that gap would make the browser submit the form itself: the
  // request never reaches the Inertia handler and the page reloads with
  // empty fields. `onMount` does not run during server rendering, so the
  // server sends the button disabled and the browser enables it after
  // hydration.
  let ready = $state(false)
  onMount(() => {
    ready = true
  })
</script>

<Button
  type="submit"
  {variant}
  {form}
  class={className}
  disabled={processing || !ready}
  aria-busy={processing}
>
  {#if processing}
    <Loader2Icon class="animate-spin" data-icon="inline-start" />
  {/if}
  {@render children()}
</Button>
