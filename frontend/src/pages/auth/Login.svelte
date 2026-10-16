<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import Head from '$lib/components/Head.svelte'
  import SubmitButton from '$lib/components/SubmitButton.svelte'
  import * as Card from '$lib/components/ui/card/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import { fieldError } from '$lib/forms.js'

  const form = useForm({
    email: '',
    password: '',
    remember: false,
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/login')
  }
</script>

<Head title="Log in" />

<Card.Root>
  <Card.Header class="text-center">
    <Card.Title class="text-xl"><h1>Sign in to Nebula</h1></Card.Title>
    <Card.Description>Welcome back. Enter your credentials to continue.</Card.Description>
  </Card.Header>
  <Card.Content>
    <form class="flex flex-col gap-5" onsubmit={submit}>
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
        <div class="flex items-center justify-between">
          <Field.Label for="password">Password</Field.Label>
          <Link href="/forgot-password" class="text-sm text-primary underline-offset-4 hover:underline">
            Forgot your password?
          </Link>
        </div>
        <Input
          id="password"
          type="password"
          autocomplete="current-password"
          required
          aria-invalid={!!form.errors.password}
          bind:value={form.password}
        />
        <Field.Error errors={fieldError(form.errors.password)} />
      </Field.Field>

      <div class="flex items-center gap-2">
        <Checkbox id="remember" bind:checked={form.remember} />
        <Label for="remember">Remember me</Label>
      </div>

      <SubmitButton class="w-full" processing={form.processing}>Sign in</SubmitButton>
    </form>
  </Card.Content>
  <Card.Footer class="justify-center text-sm text-muted-foreground">
    Don't have an account?
    <Link href="/register" class="ml-1 text-primary underline-offset-4 hover:underline">Register</Link>
  </Card.Footer>
</Card.Root>
