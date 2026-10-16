<script lang="ts">
  import { Link, page } from '@inertiajs/svelte'
  import Head from '$lib/components/Head.svelte'
  import { buttonVariants } from '$lib/components/ui/button/index.js'

  // These props come from the framework, not from a handler:
  // `InertiaConfig::error_page` in `src/bootstrap.rs` routes every framework
  // error response (403, 404, 419, 429, 500, 503) to this page, so
  // `suprnova generate-types` writes no `Pages` entry for it.
  interface ErrorProps {
    status: number
    message: string
    request_id?: string
  }

  let { status, message, request_id }: ErrorProps = $props()

  // The public root the server shares with every page (`RootShare`).
  const home = $derived(`${page.props.root}/`)
</script>

<Head title={String(status)} />

<div class="mx-auto flex max-w-md flex-col items-center gap-4 px-4 py-16 text-center">
  <h1 class="text-6xl font-extrabold">{status}</h1>
  <p class="text-lg text-muted-foreground">{message}</p>
  {#if request_id}
    <p class="text-sm text-muted-foreground">
      Reference: <code class="rounded bg-muted px-1 font-mono">{request_id}</code>
    </p>
  {/if}
  <Link href={home} class={buttonVariants({ variant: 'outline' })}>Go home</Link>
</div>
