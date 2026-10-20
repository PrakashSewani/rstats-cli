# rstats viewer

A static web viewer for [`rstats`](../README.md) recordings. Open a recording on the landing page and explore it on a `/view` route: zoomable, aligned timelines for CPU (total + per-core), memory, swap, load, network, and disk — plus a process explorer for deep-capture recordings.

Everything is client-side. Files are parsed locally in the browser and never uploaded anywhere. There is no backend.

## Inputs

- `rstats` recordings (`recordings/rstats-*.jsonl`) — standard or deep capture.
- `rstats --export ... --format json` output — the flattened per-sample rows.
- A single `--once --json` snapshot.

Formats are auto-detected. Deep recordings (which include the process table) unlock the process explorer; standard recordings and exports show charts only.

## Develop

```sh
npm install
npm run dev        # http://localhost:5173/
npm run check      # TypeScript typecheck
npm run build      # typecheck + production build into dist/
npm run preview    # serve the production build
npm run demo       # regenerate public/demo/*.jsonl fixtures
```

## Structure

```text
src/
  main.ts                 route dispatch (landing / view)
  router.ts               tiny history-API router
  state.ts                in-memory session handoff between routes
  styles.css              design system (dark, cyan accent, TUI palette)
  lib/
    parse.ts              streaming JSONL + export JSON parser (auto-detect)
    charts.ts             uPlot chart factory with shared cursor sync
    stats.ts              averages/peaks and network totals
    format.ts             byte/rate/percent/duration formatting
    dom.ts, types.ts      helpers and shared types
  pages/
    landing.ts            hero, drop zone, demo loaders, feature cards
    view.ts               KPIs, charts, process explorer
public/demo/              deterministic demo fixtures (standard + deep)
scripts/
  generate-demo.mjs       fixture generator (seeded, stable output)
  postbuild.mjs           writes dist/_redirects (SPA fallback)
```

## Deploy

The viewer is hosted on Cloudflare Pages at <https://rstats.prakashsewani.com>. It is not deployed to GitHub Pages.

Cloudflare Pages project settings:

- Project: `rstats`
- Root directory: `viewer`
- Build command: `npm ci && npm run build`
- Build output directory: `dist`
- Production branch: `main`

The project is connected to this repository, so pushes to `main` that touch `viewer/` deploy automatically. For a manual deploy from a logged-in Wrangler session:

```sh
cd viewer
npm ci
npm run build
npx wrangler pages deploy dist --project-name rstats
```

Add `rstats.prakashsewani.com` under the Pages project's **Custom domains** settings. Because `prakashsewani.com` is already in the same Cloudflare account, Pages creates the DNS record automatically; otherwise add a proxied `CNAME` from `rstats` to `rstats.pages.dev`.

`vite.config.ts` uses the root base `/`, and `postbuild.mjs` writes `_redirects` so deep links like `/view` refresh correctly.
