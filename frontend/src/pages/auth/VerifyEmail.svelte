<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import Link2OffIcon from '@lucide/svelte/icons/link-2-off'
  import MailCheckIcon from '@lucide/svelte/icons/mail-check'
  import Head from '$lib/components/Head.svelte'
  import SubmitButton from '$lib/components/SubmitButton.svelte'
  import * as Alert from '$lib/components/ui/alert/index.js'
  import * as Card from '$lib/components/ui/card/index.js'

  // `status` is set by the server: `"invalid-or-expired"` when a bad token
  // landed on the verify route, `null` otherwise. A successful resend
  // redirects back here without a status key, so the "link sent" notice keys
  // off a local flag instead.
  let { status }: { status: string | null } = $props()

  const linkInvalid = $derived(status === 'invalid-or-expired')

  const resendForm = useForm({})
  const logoutForm = useForm({})

  // Tracks "a fresh link was sent" across submits: cleared at submit start,
  // set on success (`resendForm.wasSuccessful` would stick after a failure).
  let linkSent = $state(false)

  function resend(e: SubmitEvent) {
    e.preventDefault()
    linkSent = false
    resendForm.post('/email/verification-notification', {
      preserveState: true,
      onSuccess: () => {
        linkSent = true
      },
    })
  }

  function logout(e: SubmitEvent) {
    e.preventDefault()
    logoutForm.post('/logout')
  }
</script>

<Head title="Verify email" />

<Card.Root>
  <Card.Header class="text-center">
    <Card.Title class="text-xl"><h1>Verify your email address</h1></Card.Title>
    <Card.Description>
      Before continuing, please check your inbox for a verification link. If you
      didn't receive the email, we'll gladly send you another.
    </Card.Description>
  </Card.Header>
  <Card.Content class="flex flex-col gap-5">
    {#if linkInvalid}
      <Alert.Root variant="destructive">
        <Link2OffIcon />
        <Alert.Title>
          That verification link is invalid or has expired. Request a fresh one below.
        </Alert.Title>
      </Alert.Root>
    {:else if linkSent}
      <Alert.Root>
        <MailCheckIcon />
        <Alert.Title>A fresh verification link has been sent to your email address.</Alert.Title>
      </Alert.Root>
    {/if}

    <form onsubmit={resend}>
      <SubmitButton class="w-full" processing={resendForm.processing}>
        Resend verification email
      </SubmitButton>
    </form>

    <!-- Profile stays open to unverified users, so a mistyped address can
         be corrected there (`src/routes.rs`). -->
    <p class="text-center text-sm text-muted-foreground">
      Wrong address?
      <Link href="/profile" class="text-primary underline-offset-4 hover:underline">Update it in your profile</Link>.
    </p>
  </Card.Content>
  <Card.Footer class="justify-center">
    <form onsubmit={logout}>
      <SubmitButton variant="link" processing={logoutForm.processing}>Log out</SubmitButton>
    </form>
  </Card.Footer>
</Card.Root>
