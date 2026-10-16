<script lang="ts">
  import { Link, page } from '@inertiajs/svelte'

  // These props come from the framework, not from a handler:
  // `InertiaConfig::error_page` in `src/bootstrap.rs` routes every framework
  // error response (403, 404, 419, 429, 500, 503) to this page. No handler
  // renders `Error` with a struct, so `suprnova generate-types` writes no
  // `Pages` entry for it, and the props are declared here.
  interface ErrorProps {
    status: number
    message: string
    request_id?: string
  }

  let { status, message, request_id }: ErrorProps = $props()

  // The public root the server shares with every page (`RootShare`): empty
  // at the host root, the prefix behind a proxy that serves the app there.
  const home = $derived(`${page.props.root}/`)
</script>

<svelte:head>
  <title>{status} · Nebula</title>
</svelte:head>

<div class="mx-auto max-w-md space-y-4 px-4 py-16 text-center">
  <h1 class="text-6xl font-extrabold text-on-surface">{status}</h1>
  <p class="text-lg text-on-surface-variant">{message}</p>
  {#if request_id}
    <p class="text-sm text-on-surface-variant">
      Reference:
      <code class="rounded bg-surface-container-high px-1">{request_id}</code>
    </p>
  {/if}
  <p>
    <Link href={home} class="font-medium text-primary hover:underline">Go home</Link>
  </p>
</div>
