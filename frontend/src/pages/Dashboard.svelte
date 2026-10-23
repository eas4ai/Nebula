<script lang="ts">
  import Head from '$lib/components/Head.svelte'
  import * as Card from '$lib/components/ui/card/index.js'
  import type { DashboardProps } from '../types/inertia-props'

  import { Deferred, Link, usePoll, WhenVisible } from '@inertiajs/svelte'
  import { formatDate } from '$lib/dates'
  import { useLive } from '$lib/live/index.svelte'

  useLive()
  usePoll(10000, { only: ['stats'] })

  let { user, stats, recent_notes }: DashboardProps = $props()
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
  <Card.Root>
    <Card.Header><Card.Title>Your notes</Card.Title></Card.Header>
    <Card.Content>
      <Deferred data="stats">
        {#snippet fallback()}<p>Counting your notes...</p>{/snippet}
        {#if stats}
          <p>{stats.notes} notes, {stats.written_today} written today.</p>
        {/if}
      </Deferred>
      <Link href="/notes" class="text-primary underline">Read and write notes</Link>
    </Card.Content>
  </Card.Root>
  <Card.Root>
    <Card.Header><Card.Title>Recent notes</Card.Title></Card.Header>
    <Card.Content>
      <WhenVisible data="recent_notes">
        {#snippet fallback()}<p>Loading recent notes...</p>{/snippet}
        <ul class="space-y-2">
          {#each recent_notes ?? [] as note (note.id)}
            <li><Link href={`/notes/${note.id}`} class="text-primary underline">{note.title}</Link> <time datetime={note.created_at}>{formatDate(note.created_at)}</time></li>
          {:else}<li>No notes yet.</li>{/each}
        </ul>
      </WhenVisible>
    </Card.Content>
  </Card.Root>
</div>
