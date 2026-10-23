<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import MailIcon from '@lucide/svelte/icons/mail'
  import Head from '$lib/components/Head.svelte'
  import SubmitButton from '$lib/components/SubmitButton.svelte'
  import * as Alert from '$lib/components/ui/alert/index.js'
  import * as Card from '$lib/components/ui/card/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { fieldError } from '$lib/forms.js'

  const form = useForm({
    email: '',
  })

  // Tracks "a reset link was sent" across submits. `form.wasSuccessful` sticks
  // once set, so a later failed submit would show the success notice next to
  // a field error; this flag clears at submit start and persists otherwise.
  let linkSent = $state(false)

  function submit(e: SubmitEvent) {
    e.preventDefault()
    linkSent = false
    form.post('/forgot-password', {
      preserveState: true,
      onSuccess: () => {
        linkSent = true
      },
    })
  }
</script>

<Head title="Forgot password" />

<Card.Root>
  <Card.Header class="text-center">
    <Card.Title class="text-xl"><h1>Forgot your password?</h1></Card.Title>
    <Card.Description>
      Enter your email address and we'll send you a link to reset it.
    </Card.Description>
  </Card.Header>
  <Card.Content class="flex flex-col gap-5">
    {#if linkSent}
      <Alert.Root>
        <MailIcon />
        <Alert.Title>Check your inbox</Alert.Title>
        <Alert.Description>
          If that email address is in our system, a password reset link is on its way.
        </Alert.Description>
      </Alert.Root>
    {/if}

    <form class="flex flex-col gap-5" onsubmit={submit}>
      <Field.Field data-invalid={!!form.errors.email}>
        <Field.Label for="email">Email address</Field.Label>
        <Input
          id="email"
          type="email"
          autocomplete="email"
          maxlength={254}
          placeholder="you@example.com"
          required
          aria-invalid={!!form.errors.email}
          bind:value={form.email}
        />
        <Field.Error errors={fieldError(form.errors.email)} />
      </Field.Field>

      <SubmitButton class="w-full" processing={form.processing}>Email password reset link</SubmitButton>
    </form>
  </Card.Content>
  <Card.Footer class="justify-center text-sm text-muted-foreground">
    Remembered it after all?
    <Link href="/login" class="ml-1 text-primary underline-offset-4 hover:underline">Back to sign in</Link>
  </Card.Footer>
</Card.Root>
