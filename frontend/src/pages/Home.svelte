<script lang="ts">
  import { Link, page } from '@inertiajs/svelte'
  import type { HomeProps } from '../types/inertia-props'
  import type { AuthUser } from '../types/auth'

  let { title, message }: HomeProps = $props()

  // Same auth-prop read as Layout: `page` is a $state-backed reactive object
  // in @inertiajs/svelte 3, so plain property access stays reactive.
  const user = $derived(
    (page.props.auth as { user?: AuthUser | null } | undefined)?.user ?? null,
  )
</script>

<svelte:head>
  <title>Welcome - Nebula</title>
</svelte:head>

<div
  class="mx-auto flex w-full max-w-2xl flex-col items-center px-4 py-16 text-center sm:py-24"
>
  <!--
    The full SUPRNOVA badge (apple-touch-icon is the 180px badge render -
    same artwork as android-chrome-512 at a fraction of the weight). Served
    by the kit's static_files routes at the web root.
  -->
  <img
    src="/apple-touch-icon.png"
    alt="Suprnova"
    width="180"
    height="180"
    class="size-32 shadow-lg sm:size-40"
  />

  <h1
    class="mt-8 text-4xl font-bold tracking-tight text-on-surface sm:text-5xl"
  >
    {title}
  </h1>
  <p class="mt-4 max-w-xl text-lg text-on-surface-variant">{message}</p>

  <!--
    CTAs are Inertia <Link>s styled as buttons rather than sv5ui Button+href:
    sv5ui's Button delegates href rendering to its internal SvelteKit-coupled
    Link (a plain <a> here, since $app/state is stubbed), which would do a
    full page load. Inertia <Link> keeps navigation client-side.
  -->
  <div class="mt-10 flex flex-wrap items-center justify-center gap-4">
    {#if user}
      <Link
        href="/dashboard"
        class="rounded-lg bg-primary px-5 py-2.5 text-sm font-semibold text-on-primary transition-opacity hover:opacity-90"
      >
        Go to dashboard
      </Link>
    {:else}
      <Link
        href="/register"
        class="rounded-lg bg-primary px-5 py-2.5 text-sm font-semibold text-on-primary transition-opacity hover:opacity-90"
      >
        Get started
      </Link>
      <Link
        href="/login"
        class="rounded-lg border border-outline-variant px-5 py-2.5 text-sm font-semibold text-on-surface transition-colors hover:bg-surface-container"
      >
        Sign in
      </Link>
    {/if}
  </div>

  <p class="mt-12 text-sm text-on-surface-variant">
    Edit
    <code class="bg-surface-container px-1 rounded"
      >frontend/src/pages/Home.svelte</code
    >
    to get started.
  </p>
</div>
