# useindelible.com

Public website for Indelible: brand landing page + product docs (Astro + Starlight).

## Develop

```bash
pnpm install
pnpm dev        # http://localhost:4321
pnpm check      # astro type/content checks
pnpm build      # static output in dist/
pnpm preview    # serve dist/ locally
```

Docs content lives in `src/content/docs/docs/` — the nested `docs/` directory is
what places every page under the `/docs/` URL prefix. Add markdown there; the
sidebar groups (`getting-started/`, `self-hosting/`, `how-to/`, `reference/`)
autogenerate from the directory tree.

Landing screenshots in `src/assets/screenshots/` are captured from the HTML
prototypes in `prototypes/` (app shell element at 2x). Replace with live-app
captures post-launch.

## Deploy (Cloudflare Pages)

Connected to this repo via the Cloudflare dashboard. Settings:

| Setting | Value |
| --- | --- |
| Production branch | `main` |
| Root directory | `website` |
| Framework preset | Astro |
| Build command | `pnpm build` |
| Build output directory | `dist` |

Custom domain: `useindelible.com` (+ `www` redirect). Every push to `main` that
touches `website/` redeploys; CI (`.github/workflows/website.yml`) gates merges
with `astro check` + build.
