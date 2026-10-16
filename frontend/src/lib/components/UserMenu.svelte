<script lang="ts">
  import { router } from '@inertiajs/svelte'
  import LogOutIcon from '@lucide/svelte/icons/log-out'
  import UserIcon from '@lucide/svelte/icons/user'
  import * as Avatar from '$lib/components/ui/avatar/index.js'
  import { buttonVariants } from '$lib/components/ui/button/index.js'
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js'

  let { name }: { name: string } = $props()

  // Up to two initials from the user's name, for the avatar.
  const initials = $derived(
    name
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((word) => word[0]!.toUpperCase())
      .join('') || '?',
  )
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger class={buttonVariants({ variant: 'ghost' })} aria-label="Account menu">
    <Avatar.Root class="size-6">
      <Avatar.Fallback class="text-xs">{initials}</Avatar.Fallback>
    </Avatar.Root>
    <span class="hidden sm:inline">{name}</span>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-44">
    <DropdownMenu.Item onSelect={() => router.visit('/profile')}>
      <UserIcon />
      Profile
    </DropdownMenu.Item>
    <DropdownMenu.Separator />
    <DropdownMenu.Item variant="destructive" onSelect={() => router.post('/logout')}>
      <LogOutIcon />
      Log out
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
