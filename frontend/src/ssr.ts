import { createInertiaApp } from '@inertiajs/svelte'
import AppLayout from '$lib/layouts/AppLayout.svelte'
import GuestLayout from '$lib/layouts/GuestLayout.svelte'

// `suprnova ssr:start` runs this entry's bundle under Node, and
// `npm run build:ssr` (`vite build --ssr src/ssr.ts`) produces it at
// `bootstrap/ssr/ssr.js`. On an SSR build `@inertiajs/vite` wraps this
// top-level `createInertiaApp(...)` call in the server bootstrap: it renders
// each page the framework posts to `POST /render` and answers `GET /health`
// for `suprnova ssr:check`. Under `npm run dev` the plugin serves the same
// render at `/__inertia_ssr` instead.
//
// The options match `main.ts`, so the markup the server renders is the
// markup the browser hydrates.
createInertiaApp({
  pages: './pages',
  layout: (name) => (name.startsWith('auth/') ? GuestLayout : AppLayout),
  serverHead: true,
})
