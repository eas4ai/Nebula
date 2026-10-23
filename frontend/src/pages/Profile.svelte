<script lang="ts">
  import { untrack } from 'svelte'
  import { useForm } from '@inertiajs/svelte'
  import BadgeCheckIcon from '@lucide/svelte/icons/badge-check'
  import CircleAlertIcon from '@lucide/svelte/icons/circle-alert'
  import Head from '$lib/components/Head.svelte'
  import SubmitButton from '$lib/components/SubmitButton.svelte'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Card from '$lib/components/ui/card/index.js'
  import * as Dialog from '$lib/components/ui/dialog/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { fieldError } from '$lib/forms.js'

  let { name, email, email_verified }: {
    name: string
    email: string
    email_verified: boolean
  } = $props()

  // Seed the profile form from the server-provided values: they are the
  // initial state of this page.
  const profileForm = useForm({
    name: untrack(() => name),
    email: untrack(() => email),
  })

  const passwordForm = useForm({
    current_password: '',
    password: '',
    password_confirmation: '',
  })

  const deleteForm = useForm({
    password: '',
  })

  let confirmingDeletion = $state(false)

  function submitProfile(e: SubmitEvent) {
    e.preventDefault()
    profileForm.patch('/profile')
  }

  function submitPassword(e: SubmitEvent) {
    e.preventDefault()
    passwordForm.put('/profile/password', {
      onSuccess: () => passwordForm.reset(),
    })
  }

  function submitDelete(e: SubmitEvent) {
    e.preventDefault()
    deleteForm.delete('/profile')
  }

  // Closing the dialog by any path (Cancel, Escape, the overlay) clears it.
  function onDialogOpenChange(open: boolean) {
    if (!open) {
      deleteForm.reset()
      deleteForm.clearErrors()
    }
  }
</script>

<Head title="Profile" />

<div class="mx-auto flex w-full max-w-2xl flex-col gap-6 px-4 py-10">
  <h1 class="text-2xl font-semibold">Profile</h1>

  <!--
    Each card reads its own form's `errors` (filled only when that form
    submitted), so a 422 from one action never shows on another card.
  -->
  <Card.Root>
    <Card.Header>
      <Card.Title><h2>Profile information</h2></Card.Title>
      <Card.Description>Update your account's name and email address.</Card.Description>
      <Card.Action>
        {#if email_verified}
          <Badge variant="secondary"><BadgeCheckIcon />Email verified</Badge>
        {:else}
          <Badge variant="destructive"><CircleAlertIcon />Email not verified</Badge>
        {/if}
      </Card.Action>
    </Card.Header>
    <Card.Content>
      <form class="flex flex-col gap-5" onsubmit={submitProfile}>
        <Field.Field data-invalid={!!profileForm.errors.name}>
          <Field.Label for="name">Name</Field.Label>
          <Input
            id="name"
            type="text"
            autocomplete="name"
          maxlength={255}
            required
            aria-invalid={!!profileForm.errors.name}
            bind:value={profileForm.name}
          />
          <Field.Error errors={fieldError(profileForm.errors.name)} />
        </Field.Field>

        <Field.Field data-invalid={!!profileForm.errors.email}>
          <Field.Label for="email">Email address</Field.Label>
          <Input
            id="email"
            type="email"
            autocomplete="email"
          maxlength={254}
            required
            aria-invalid={!!profileForm.errors.email}
            bind:value={profileForm.email}
          />
          <Field.Description>Changing your email requires verifying the new address.</Field.Description>
          <Field.Error errors={fieldError(profileForm.errors.email)} />
        </Field.Field>

        <div class="flex items-center gap-3">
          <SubmitButton processing={profileForm.processing}>Save</SubmitButton>
          {#if profileForm.recentlySuccessful}
            <span class="text-sm text-muted-foreground">Saved.</span>
          {/if}
        </div>
      </form>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title><h2>Update password</h2></Card.Title>
      <Card.Description>Use a long, random password to keep your account secure.</Card.Description>
    </Card.Header>
    <Card.Content>
      <form class="flex flex-col gap-5" onsubmit={submitPassword}>
        <Field.Field data-invalid={!!passwordForm.errors.current_password}>
          <Field.Label for="current_password">Current password</Field.Label>
          <Input
            id="current_password"
            type="password"
            autocomplete="current-password"
            required
            aria-invalid={!!passwordForm.errors.current_password}
            bind:value={passwordForm.current_password}
          />
          <Field.Error errors={fieldError(passwordForm.errors.current_password)} />
        </Field.Field>

        <Field.Field data-invalid={!!passwordForm.errors.password}>
          <Field.Label for="new_password">New password</Field.Label>
          <Input
            id="new_password"
            type="password"
            autocomplete="new-password"
            required
            aria-invalid={!!passwordForm.errors.password}
            bind:value={passwordForm.password}
          />
          <Field.Description>At least 8 characters.</Field.Description>
          <Field.Error errors={fieldError(passwordForm.errors.password)} />
        </Field.Field>

        <Field.Field data-invalid={!!passwordForm.errors.password_confirmation}>
          <Field.Label for="new_password_confirmation">Confirm new password</Field.Label>
          <Input
            id="new_password_confirmation"
            type="password"
            autocomplete="new-password"
            required
            aria-invalid={!!passwordForm.errors.password_confirmation}
            bind:value={passwordForm.password_confirmation}
          />
          <Field.Error errors={fieldError(passwordForm.errors.password_confirmation)} />
        </Field.Field>

        <div class="flex items-center gap-3">
          <SubmitButton processing={passwordForm.processing}>Update password</SubmitButton>
          {#if passwordForm.recentlySuccessful}
            <span class="text-sm text-muted-foreground">Password updated.</span>
          {/if}
        </div>
      </form>
    </Card.Content>
  </Card.Root>

  <Card.Root class="border-destructive/40">
    <Card.Header>
      <Card.Title class="text-destructive"><h2>Delete account</h2></Card.Title>
      <Card.Description>
        Once your account is deleted, all of its data is permanently removed.
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <Dialog.Root bind:open={confirmingDeletion} onOpenChange={onDialogOpenChange}>
        <Dialog.Trigger>
          {#snippet child({ props })}
            <Button variant="destructive" {...props}>Delete account</Button>
          {/snippet}
        </Dialog.Trigger>
        <Dialog.Content>
          <Dialog.Header>
            <Dialog.Title>Delete account</Dialog.Title>
            <Dialog.Description>
              This action cannot be undone. Enter your password to confirm you
              want to permanently delete your account.
            </Dialog.Description>
          </Dialog.Header>
          <form id="delete-account-form" onsubmit={submitDelete}>
            <Field.Field data-invalid={!!deleteForm.errors.password}>
              <Field.Label for="delete_password">Password</Field.Label>
              <Input
                id="delete_password"
                type="password"
                autocomplete="current-password"
                placeholder="Your current password"
                required
                aria-invalid={!!deleteForm.errors.password}
                bind:value={deleteForm.password}
              />
              <Field.Error errors={fieldError(deleteForm.errors.password)} />
            </Field.Field>
          </form>
          <Dialog.Footer>
            <Dialog.Close>
              {#snippet child({ props })}
                <Button variant="outline" {...props}>Cancel</Button>
              {/snippet}
            </Dialog.Close>
            <SubmitButton
              form="delete-account-form"
              variant="destructive"
              processing={deleteForm.processing}
            >
              Permanently delete
            </SubmitButton>
          </Dialog.Footer>
        </Dialog.Content>
      </Dialog.Root>
    </Card.Content>
  </Card.Root>
</div>
