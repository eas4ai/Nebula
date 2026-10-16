<script lang="ts">
  import type { Snippet } from 'svelte'
  import { Link } from '@inertiajs/svelte'
  import { ModeWatcher } from 'mode-watcher'
  import FlashToaster from '$lib/components/FlashToaster.svelte'
  import ModeToggle from '$lib/components/ModeToggle.svelte'
  import NebulaMark from '$lib/components/NebulaMark.svelte'

  // The frame of the sign-in pages (`auth/*`): the Nebula mark above a
  // centred card, as in shadcn-svelte's login blocks.
  let { children }: { children: Snippet } = $props()
</script>

<ModeWatcher defaultMode="dark" disableHeadScriptInjection />
<FlashToaster />

<div class="relative flex min-h-svh flex-col items-center justify-center gap-6 p-4 sm:p-6">
  <div class="absolute top-4 right-4">
    <ModeToggle />
  </div>
  <Link href="/" class="flex items-center gap-2 font-semibold">
    <NebulaMark class="size-8" />
    <span class="text-lg">Nebula</span>
  </Link>
  <main class="w-full max-w-sm">
    {@render children()}
  </main>
</div>
