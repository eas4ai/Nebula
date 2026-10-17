<script lang="ts">
  import { Link, page } from '@inertiajs/svelte'
  import Head from '$lib/components/Head.svelte'
  import { buttonVariants } from '$lib/components/ui/button/index.js'
  import type { HomeProps } from '../types/inertia-props'

  let { title, message }: HomeProps = $props()

  const user = $derived(page.props.auth?.user ?? null)
</script>

<Head title="Welcome" />

<div class="mx-auto flex w-full max-w-2xl flex-col items-center px-4 py-16 text-center sm:py-24">
  <!-- The full Suprnova badge (apple-touch-icon is the 180px render). -->
  <img
    src="/apple-touch-icon.png"
    alt="Suprnova"
    width="180"
    height="180"
    class="size-32 rounded-2xl shadow-lg sm:size-40"
  />

  <h1 class="mt-8 text-4xl font-bold tracking-tight sm:text-5xl">{title}</h1>
  <p class="mt-4 max-w-xl text-lg text-muted-foreground">{message}</p>

  <!-- Inertia <Link>s styled as buttons keep navigation client-side. -->
  <div class="mt-10 flex flex-wrap items-center justify-center gap-3">
    {#if user}
      <Link href="/dashboard" class={buttonVariants({ size: 'lg' })}>Go to dashboard</Link>
    {:else}
      <Link href="/register" class={buttonVariants({ size: 'lg' })}>Get started</Link>
      <Link href="/login" class={buttonVariants({ variant: 'outline', size: 'lg' })}>Sign in</Link>
    {/if}
  </div>

  <p class="mt-12 text-sm text-muted-foreground">
    Edit
    <code class="rounded bg-muted px-1 font-mono">frontend/src/pages/Home.svelte</code>
    to get started.
  </p>
</div>
