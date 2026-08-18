<script lang="ts">
  import { untrack } from 'svelte'
  import { Link, useForm } from '@inertiajs/svelte'
  import { Alert, Button, Card, FormField, Input } from 'sv5ui'
  import AuthEmblem from '../../lib/AuthEmblem.svelte'

  let { token }: { token: string } = $props()

  // The reset token is fixed for the life of this page (it comes from the
  // one-shot emailed link), so seed the form with its initial value. `untrack`
  // makes that intent explicit and silences the state-referenced-locally hint.
  const form = useForm({
    token: untrack(() => token),
    password: '',
    password_confirmation: '',
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/reset-password')
  }

  // A `token` error means the link is consumed or expired - no resubmit with
  // this token can ever succeed, so the form is replaced with a CTA to
  // request a fresh link.
  const tokenError = $derived(form.errors.token?.[0] ?? null)
</script>

<div class="flex justify-center px-4 py-12 sm:py-16">
  <Card class="w-full max-w-md">
    {#snippet header()}
      <AuthEmblem />
      <h1 class="mt-3 text-center text-xl font-semibold text-on-surface">
        Reset your password
      </h1>
      <p class="mt-1 text-center text-sm text-on-surface-variant">
        Choose a new password for your account.
      </p>
    {/snippet}

    <div class="space-y-5">
      {#if tokenError}
        <Alert
          color="error"
          variant="soft"
          icon="lucide:link-2-off"
          title={tokenError}
        />

        <Link
          href="/forgot-password"
          class="block rounded-lg bg-primary px-3 py-2 text-center text-sm font-medium text-on-primary transition-opacity hover:opacity-90"
        >
          Request a new link
        </Link>
      {:else}
        <form class="space-y-5" onsubmit={submit}>
          <FormField
            name="password"
            label="New password"
            required
            help="At least 8 characters."
            error={form.errors.password?.[0]}
          >
            <Input
              type="password"
              autocomplete="new-password"
              required
              bind:value={form.password}
            />
          </FormField>

          <FormField
            name="password_confirmation"
            label="Confirm new password"
            required
            error={form.errors.password_confirmation?.[0]}
          >
            <Input
              type="password"
              autocomplete="new-password"
              required
              bind:value={form.password_confirmation}
            />
          </FormField>

          <Button
            type="submit"
            block
            label="Reset password"
            loading={form.processing}
          />
        </form>
      {/if}
    </div>

    {#snippet footer()}
      <p class="text-center text-sm text-on-surface-variant">
        <Link href="/login" class="font-medium text-primary hover:underline">
          Back to sign in
        </Link>
      </p>
    {/snippet}
  </Card>
</div>
