import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import inertia from '@inertiajs/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig(({ isSsrBuild }) => ({
  // Relative, so the built entry loads its chunks and assets from where the
  // server's tags put it: the same build runs at `/` and behind a proxy that
  // serves the application under a path prefix.
  base: './',
  // The application's root .env owns public Vite settings (such as
  // `VITE_REPORTING_DSN`) as well as server config.
  envDir: '..',
  plugins: [
    tailwindcss(),
    // The Inertia plugin turns `pages: './pages'` in both entries into the
    // page resolver and wraps `src/ssr.ts` into the SSR server on an SSR
    // build. Declaring it is also what makes `suprnova serve` write the
    // `public/hot` file, so development pages load from this dev server
    // instead of an earlier `npm run build`. Under `npm run dev` it serves
    // the SSR entry at `/__inertia_ssr`. `suprnova ssr:start` runs Node
    // without `--enable-source-maps`, so no SSR source map is written.
    inertia({ ssr: { entry: 'src/ssr.ts', sourcemap: false } }),
    svelte(),
  ],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
    },
  },
  server: {
    // `suprnova serve` sets VITE_PORT to the port it resolved (the
    // distinctive 5765 default, or a scanned free port). Falling back to
    // 5765 keeps a bare `npm run dev` off the squatted 5173.
    port: Number(process.env.VITE_PORT) || 5765,
    strictPort: true,
    cors: true,
  },
  build: isSsrBuild
    ? {
        // `vite build --ssr src/ssr.ts` lands here, not in `public/assets`:
        // `suprnova ssr:start` looks for `frontend/bootstrap/ssr/ssr.js`.
        outDir: 'bootstrap/ssr',
        rollupOptions: {
          output: {
            // Pin the filename; Vite's default naming for an SSR entry is
            // not a contract `ssr:start` can rely on.
            entryFileNames: 'ssr.js',
          },
        },
      }
    : {
        outDir: '../public/assets',
        manifest: true,
        rollupOptions: {
          input: 'src/main.ts',
        },
      },
}))
