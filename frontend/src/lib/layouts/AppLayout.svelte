<script lang="ts">
  import type { Snippet } from 'svelte'
  import { Link, page } from '@inertiajs/svelte'
  import { ModeWatcher } from 'mode-watcher'
  import FlashToaster from '$lib/components/FlashToaster.svelte'
  import ModeToggle from '$lib/components/ModeToggle.svelte'
  import NebulaMark from '$lib/components/NebulaMark.svelte'
  import UserMenu from '$lib/components/UserMenu.svelte'
  import { buttonVariants } from '$lib/components/ui/button/index.js'

  let { children }: { children: Snippet } = $props()

  // The shared `auth` prop (`src/props/shared.rs`), typed by the generated
  // `SharedProps`. `page` is $state-backed, so this stays reactive.
  const user = $derived(page.props.auth?.user ?? null)
</script>

<!-- The script that sets the mode before first paint is in
     `templates/app.html`, which receives the request's CSP nonce. -->
<ModeWatcher defaultMode="dark" disableHeadScriptInjection />
<FlashToaster />

<div class="flex min-h-svh flex-col">
  <header class="border-b">
    <nav class="mx-auto flex h-14 w-full max-w-5xl items-center gap-2 px-4 sm:gap-4">
      <Link href="/" class="flex items-center gap-2 font-semibold">
        <NebulaMark />
        <span>Nebula</span>
      </Link>

      {#if user}
        <Link
          href="/dashboard"
          class="text-sm font-medium text-muted-foreground transition-colors hover:text-foreground"
        >
          Dashboard
        </Link>
        <Link href="/notes" class="text-sm font-medium text-muted-foreground hover:text-foreground">Notes</Link>
        <Link href="/connected-agents" class="text-sm font-medium text-muted-foreground hover:text-foreground">Connected agents</Link>
      {/if}

      <div class="ml-auto flex items-center gap-1 sm:gap-2">
        <ModeToggle />
        {#if user}
          <UserMenu name={user.name} />
        {:else}
          <Link href="/login" class={buttonVariants({ variant: 'ghost' })}>Log in</Link>
          <Link href="/register" class={buttonVariants({ variant: 'default' })}>Register</Link>
        {/if}
      </div>
    </nav>
  </header>

  <main class="flex-1">
    {@render children()}
  </main>
</div>
