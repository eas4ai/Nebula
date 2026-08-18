<script lang="ts">
  import type { Snippet } from 'svelte'
  import { Link, page, router } from '@inertiajs/svelte'
  import { ModeWatcher } from 'mode-watcher'
  import { Alert, Button, DropdownMenu, ThemeModeButton } from 'sv5ui'
  import type { DropdownMenuItem } from 'sv5ui'
  import type { AuthUser } from '../types/auth'

  let { children }: { children: Snippet } = $props()

  // `page` in @inertiajs/svelte 3 is a $state-backed reactive object (not a
  // store), so plain property access stays reactive across visits.
  const user = $derived(
    (page.props.auth as { user?: AuthUser | null } | undefined)?.user ?? null,
  )

  // `flash` is a first-class typed property on Inertia's Page object,
  // populated from the server-side session flash for this visit: the server
  // consumes the flash, the client keeps it for the rendered page. Its shape
  // comes from the `flashDataType` augmentation in `types/inertia.d.ts`.
  const flashSuccess = $derived(page.flash?.success ?? null)
  const flashError = $derived(page.flash?.error ?? null)

  const userMenuItems: DropdownMenuItem[] = [
    {
      label: 'Profile',
      icon: 'lucide:user',
      onSelect: () => router.visit('/profile'),
    },
    { type: 'separator' },
    {
      label: 'Log out',
      icon: 'lucide:log-out',
      color: 'error',
      onSelect: () => router.post('/logout'),
    },
  ]
</script>

<!--
  @inertiajs/svelte 3 ships no <Head> component (its dist exports App, Link,
  Form, Deferred, ...), so head management is Svelte's native <svelte:head>.
  This layout persists across visits, making the title below the app-wide
  default; a page can override it with its own <svelte:head><title>.
  The icon files are served by the kit's static_files routes at the web root.
-->
<svelte:head>
  <title>Nebula</title>
  <link rel="icon" href="/favicon.ico" />
  <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
  <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
  <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
  <!--
    `use-credentials` matters: browsers fetch web-app manifests WITHOUT
    cookies by default, and every Suprnova request without a session cookie
    mints a fresh session whose Set-Cookie clobbers the real one - breaking
    the CSRF token pairing on the user's very first form submit.
  -->
  <link rel="manifest" href="/site.webmanifest" crossorigin="use-credentials" />
</svelte:head>

<ModeWatcher defaultMode="dark" />

<div class="flex min-h-screen flex-col bg-surface text-on-surface">
  <header class="border-b border-outline-variant bg-surface-container-low">
    <nav class="mx-auto flex h-14 w-full max-w-5xl items-center gap-6 px-4">
      <Link href="/" class="flex items-center gap-2">
        <img
          src="/favicon-32x32.png"
          alt=""
          width="32"
          height="32"
          class="size-6"
        />
        <span class="text-lg font-semibold tracking-tight">Nebula</span>
      </Link>

      {#if user}
        <Link
          href="/dashboard"
          class="text-sm font-medium text-on-surface-variant transition-colors hover:text-on-surface"
        >
          Dashboard
        </Link>
      {/if}

      <div class="ml-auto flex items-center gap-2">
        <ThemeModeButton />

        {#if user}
          <DropdownMenu items={userMenuItems} align="end">
            {#snippet children({ props })}
              <Button
                {...props}
                variant="ghost"
                color="surface"
                trailingIcon="lucide:chevron-down"
                label={user?.name ?? 'Account'}
              />
            {/snippet}
          </DropdownMenu>
        {:else}
          <Link
            href="/login"
            class="text-sm font-medium text-on-surface-variant transition-colors hover:text-on-surface"
          >
            Log in
          </Link>
          <Link
            href="/register"
            class="rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-on-primary transition-opacity hover:opacity-90"
          >
            Register
          </Link>
        {/if}
      </div>
    </nav>
  </header>

  <!--
    Flash region renders human-ready copy only. Machine `status` keys (e.g.
    `invalid-or-expired`) are mapped to copy by the page that consumes them,
    never echoed raw here.
  -->
  {#if flashSuccess || flashError}
    <div class="mx-auto w-full max-w-5xl space-y-2 px-4 pt-4">
      {#if flashSuccess}
        <Alert color="success" variant="soft" title={flashSuccess} />
      {/if}
      {#if flashError}
        <Alert color="error" variant="soft" title={flashError} />
      {/if}
    </div>
  {/if}

  <main class="flex-1">
    {@render children()}
  </main>
</div>
