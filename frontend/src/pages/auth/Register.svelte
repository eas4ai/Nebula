<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import Head from '$lib/components/Head.svelte'
  import SubmitButton from '$lib/components/SubmitButton.svelte'
  import * as Card from '$lib/components/ui/card/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { fieldError } from '$lib/forms.js'

  const form = useForm({
    name: '',
    email: '',
    password: '',
    password_confirmation: '',
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/register')
  }
</script>

<Head title="Register" />

<Card.Root>
  <Card.Header class="text-center">
    <Card.Title class="text-xl">Create your account</Card.Title>
    <Card.Description>Join Nebula - it only takes a minute.</Card.Description>
  </Card.Header>
  <Card.Content>
    <form class="flex flex-col gap-5" onsubmit={submit}>
      <Field.Field data-invalid={!!form.errors.name}>
        <Field.Label for="name">Name</Field.Label>
        <Input
          id="name"
          type="text"
          autocomplete="name"
          placeholder="Your name"
          required
          aria-invalid={!!form.errors.name}
          bind:value={form.name}
        />
        <Field.Error errors={fieldError(form.errors.name)} />
      </Field.Field>

      <Field.Field data-invalid={!!form.errors.email}>
        <Field.Label for="email">Email address</Field.Label>
        <Input
          id="email"
          type="email"
          autocomplete="email"
          placeholder="you@example.com"
          required
          aria-invalid={!!form.errors.email}
          bind:value={form.email}
        />
        <Field.Error errors={fieldError(form.errors.email)} />
      </Field.Field>

      <Field.Field data-invalid={!!form.errors.password}>
        <Field.Label for="password">Password</Field.Label>
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
        <Field.Label for="password_confirmation">Confirm password</Field.Label>
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

      <SubmitButton class="w-full" processing={form.processing}>Register</SubmitButton>
    </form>
  </Card.Content>
  <Card.Footer class="justify-center text-sm text-muted-foreground">
    Already have an account?
    <Link href="/login" class="ml-1 text-primary underline-offset-4 hover:underline">Sign in</Link>
  </Card.Footer>
</Card.Root>
