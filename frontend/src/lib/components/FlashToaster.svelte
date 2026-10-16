<script lang="ts">
  import { page } from '@inertiajs/svelte'
  import { toast } from 'svelte-sonner'
  import { Toaster } from '$lib/components/ui/sonner/index.js'

  // `page.flash` holds what the server flashed for this response only
  // (`redirect!(...).with("success", ...)`); the next response no longer
  // has it. Each response replaces `page.flash`, so this effect runs once
  // per response and shows its message once. Effects do not run during
  // server rendering, so the toast appears after hydration.
  $effect(() => {
    const flash = page.flash
    if (flash?.success) toast.success(flash.success)
    if (flash?.error) toast.error(flash.error)
  })
</script>

<Toaster position="top-center" richColors />
