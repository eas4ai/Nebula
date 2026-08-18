<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import { Button, Card, FormField, Input } from 'sv5ui'
  import AuthEmblem from '../../lib/AuthEmblem.svelte'

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

<div class="flex justify-center px-4 py-12 sm:py-16">
  <Card class="w-full max-w-md">
    {#snippet header()}
      <AuthEmblem />
      <h1 class="mt-3 text-center text-xl font-semibold text-on-surface">
        Create your account
      </h1>
      <p class="mt-1 text-center text-sm text-on-surface-variant">
        Join Nebula - it only takes a minute.
      </p>
    {/snippet}

    <form class="space-y-5" onsubmit={submit}>
      <FormField
        name="name"
        label="Name"
        required
        error={form.errors.name?.[0]}
      >
        <Input
          type="text"
          autocomplete="name"
          placeholder="Your name"
          required
          bind:value={form.name}
        />
      </FormField>

      <FormField
        name="email"
        label="Email address"
        required
        error={form.errors.email?.[0]}
      >
        <Input
          type="email"
          autocomplete="email"
          placeholder="you@example.com"
          required
          bind:value={form.email}
        />
      </FormField>

      <FormField
        name="password"
        label="Password"
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
        label="Confirm password"
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

      <Button type="submit" block label="Register" loading={form.processing} />
    </form>

    {#snippet footer()}
      <p class="text-center text-sm text-on-surface-variant">
        Already have an account?
        <Link href="/login" class="font-medium text-primary hover:underline">
          Sign in
        </Link>
      </p>
    {/snippet}
  </Card>
</div>
