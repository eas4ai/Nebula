// sv5ui's barrel index re-exports its SvelteKit-coupled `Link`, whose
// `import { page } from '$app/state'` must resolve at bundle time. The
// `$app/state` alias in vite.config.ts points here so the resolver is
// satisfied. This stub and sv5ui's Link remain in the bundle (not tree-shaken)
// but are inert because nothing imports sv5ui's Link - Nebula navigates with
// Inertia's <Link>. The shape mirrors the `page.url` access sv5ui's Link
// performs for active-link detection.
export const page = {
  url: new URL(
    typeof window === 'undefined' ? 'http://localhost/' : window.location.href,
  ),
}
