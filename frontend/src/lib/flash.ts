import { router } from '@inertiajs/svelte'
import { toast } from 'svelte-sonner'

/**
 * Shows what the server flashes (`redirect!(...).with("success", ...)`) as a
 * toast, once per response that carries it.
 *
 * Inertia fires its `flash` event once for each such response, including the
 * first page of a full load, and not for Back/Forward or deferred-prop
 * requests. Watching `page.flash` instead would show a flash again whenever
 * Inertia reassigns the page object with the same flash. The toasts live in
 * svelte-sonner's global state, so the `<Toaster>` of whichever layout is
 * mounted shows them, also across a switch between the guest and app layouts.
 *
 * Call once, in the browser entry, before `createInertiaApp`, so the first
 * page's event is not missed.
 */
export function showFlashAsToasts(): void {
  router.on('flash', (event) => {
    const { success, error, toast: message } = event.detail.flash
    if (success) toast.success(success)
    if (error) toast.error(error)
    if (message?.kind === 'success') toast.success(message.message)
    else if (message?.kind === 'error') toast.error(message.message)
    else if (message) toast.info(message.message)
  })
}
