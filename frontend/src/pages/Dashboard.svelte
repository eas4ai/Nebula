<script lang="ts">
  import Head from '$lib/components/Head.svelte'
  import * as Card from '$lib/components/ui/card/index.js'
  import type { DashboardProps } from '../types/inertia-props'

  let { user }: DashboardProps = $props()
</script>

<Head title="Dashboard" />

<div class="mx-auto flex w-full max-w-5xl flex-col gap-6 px-4 py-10">
  <h1 class="text-2xl font-semibold">Dashboard</h1>

  <Card.Root>
    <Card.Header>
      <Card.Title><h2>Welcome, {user.name}!</h2></Card.Title>
      <Card.Description>You are logged in.</Card.Description>
    </Card.Header>
    <Card.Content class="text-sm text-muted-foreground">Email: {user.email}</Card.Content>
  </Card.Root>
</div>
