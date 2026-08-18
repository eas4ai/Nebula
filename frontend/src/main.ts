import './app.css'
import { createInertiaApp, type ResolvedComponent } from '@inertiajs/svelte'
import { hydrate, mount } from 'svelte'
import Layout from './lib/Layout.svelte'

// CSRF: no manual wiring needed. Inertia v3's HTTP client reads the live
// `XSRF-TOKEN` cookie (set by Suprnova's CsrfMiddleware, rotated on
// login/logout) and echoes it as `X-XSRF-TOKEN` on every request itself.
// Do NOT add a `router.on('before')` hook that sets the header too -
// XMLHttpRequest *combines* duplicate headers into "token, token", which
// the server rejects as a mismatch (419) on every form submit.

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob<ResolvedComponent>('./pages/**/*.svelte', {
      eager: true,
    })
    return pages[`./pages/${name}.svelte`]
  },
  // Persistent default layout. Inertia 3's `layout` option is a callback
  // `(name, page) => Component` consumed as App's `defaultLayout`; a page can
  // opt out or override by exporting its own `layout` from <script module>.
  // (Mutating `page.default.layout` in `resolve` is not an option here:
  // eager `import.meta.glob` returns frozen module namespace objects.)
  layout: () => Layout,
  setup({ el, App, props }) {
    if (el?.hasAttribute('data-server-rendered')) {
      hydrate(App, { target: el, props })
    } else {
      mount(App, { target: el!, props })
    }
  },
})
