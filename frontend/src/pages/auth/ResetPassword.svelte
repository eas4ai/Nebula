<script lang="ts">
  import { untrack } from 'svelte'
  import { Link, useForm } from '@inertiajs/svelte'
  import Link2OffIcon from '@lucide/svelte/icons/link-2-off'
  import Head from '$lib/components/Head.svelte'
  import SubmitButton from '$lib/components/SubmitButton.svelte'
  import * as Alert from '$lib/components/ui/alert/index.js'
  import { buttonVariants } from '$lib/components/ui/button/index.js'
  import * as Card from '$lib/components/ui/card/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { fieldError } from '$lib/forms.js'

  let { token }: { token: string } = $props()

  // The reset token is fixed for the life of this page (it comes from the
  // one-shot emailed link), so seed the form with its initial value.
  const form = useForm({
    token: untrack(() => token),
    password: '',
    password_confirmation: '',
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/reset-password')
  }

  // A `token` error means the link is used or expired - no resubmit with this
  // token can succeed, so the form is replaced with a link to request a new one.
  const tokenError = $derived(form.errors.token?.[0] ?? null)
</script>

<Head title="Reset password" />

<Card.Root>
  <Card.Header class="text-center">
    <Card.Title class="text-xl"><h1>Reset your password</h1></Card.Title>
    <Card.Description>Choose a new password for your account.</Card.Description>
  </Card.Header>
  <Card.Content class="flex flex-col gap-5">
    {#if tokenError}
      <Alert.Root variant="destructive">
        <Link2OffIcon />
        <Alert.Title>{tokenError}</Alert.Title>
      </Alert.Root>
      <Link href="/forgot-password" class={buttonVariants({ class: 'w-full' })}>
        Request a new link
      </Link>
    {:else}
      <form class="flex flex-col gap-5" onsubmit={submit}>
        <Field.Field data-invalid={!!form.errors.password}>
          <Field.Label for="password">New password</Field.Label>
          <Input
            id="password"
            type="password"
            autocomplete="new-password"
            required
            aria-invalid={!!form.errors.password}
            bind:value={form.password}
          />
          <Field.Description>At least 8 characters.</Field.Description>
          <Field.Error errors={fieldError(form.errors.password)} />
        </Field.Field>

        <Field.Field data-invalid={!!form.errors.password_confirmation}>
          <Field.Label for="password_confirmation">Confirm new password</Field.Label>
          <Input
            id="password_confirmation"
            type="password"
            autocomplete="new-password"
            required
            aria-invalid={!!form.errors.password_confirmation}
            bind:value={form.password_confirmation}
          />
          <Field.Error errors={fieldError(form.errors.password_confirmation)} />
        </Field.Field>

        <SubmitButton class="w-full" processing={form.processing}>Reset password</SubmitButton>
      </form>
    {/if}
  </Card.Content>
  <Card.Footer class="justify-center text-sm">
    <Link href="/login" class="text-primary underline-offset-4 hover:underline">Back to sign in</Link>
  </Card.Footer>
</Card.Root>
