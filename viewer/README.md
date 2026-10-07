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
npm run dev        # http://localhost:5173/rstats-cli/
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
  postbuild.mjs           copies dist/index.html → dist/404.html (SPA fallback)
```

## Deploy

`.github/workflows/pages.yml` builds this directory and deploys `viewer/dist` to GitHub Pages on pushes to `main` that touch `viewer/`. The repository's Pages source must be set to **GitHub Actions**.

The site is served from the project path `/rstats-cli/`; `vite.config.ts` sets the matching `base`, and the router derives routes from `import.meta.env.BASE_URL`. A `404.html` fallback is emitted so deep links like `/rstats-cli/view` refresh correctly.
