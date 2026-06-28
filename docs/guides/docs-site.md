---
title: Docs site workflow
description: Previewing and building the Tandem documentation site locally.
---

# Docs site workflow

Tandem documentation uses two directories:

- `docs/` is the canonical Markdown source.
- `site/` is the Astro Starlight project that renders and builds the static site.

Do not edit generated Markdown copies under `site/src/content/docs/`. Run the sync step after changing `docs/`.

## Install dependencies

From the repository root:

```sh
cd site
npm install
```

## Preview locally

```sh
cd site
npm run dev
```

The `predev` hook syncs `../docs/` into Starlight before the dev server starts.

## Build static output

```sh
cd site
npm run build
```

The `prebuild` hook runs `npm run sync:docs`, then Astro writes static output to `site/dist/`.

## Manual sync

```sh
cd site
npm run sync:docs
```

Use this when you want to inspect the generated Starlight content before previewing or building.

## Maintenance notes

- Commit source changes under `docs/` and site tooling changes under `site/`.
- Do not commit `site/dist/`, `site/.astro/`, or `site/node_modules/`.
- Keep generated `site/src/content/docs/**/*.md` out of version control; it exists only to bridge canonical docs into Starlight.
