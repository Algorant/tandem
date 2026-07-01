---
title: Docs site workflow
description: Previewing and building the Tandem documentation site locally.
---

# Docs site workflow

Tandem documentation uses two directories:

- `docs/` is the canonical Markdown source.
- `site/` is the Astro Starlight project that renders and builds the static site.

Do not edit generated Markdown copies under `site/src/content/docs/`. Run the sync step after changing `docs/`.

## Runtime and package manager policy

The docs site should use a supported even-numbered Node.js LTS runtime, not an arbitrary older pin. The current Astro/Starlight dependency set resolves Astro `7.0.3`, whose published `engines` require Node.js `>=22.12.0`. Astro's install docs also state that Astro requires Node.js `v22.12.0 or higher` and does not support odd-numbered Node.js releases. Node's release policy says production use should stay on Active LTS or Maintenance LTS lines; as of 2026-06-30, Node 20 is EOL, while Node 22 and Node 24 are LTS.

The GitHub Pages workflow and local shortcuts read `site/.node-version`, currently `24`. Node 24 is the current LTS line and satisfies Astro's `>=22.12.0` requirement without pinning to an obsolete or odd-numbered release. Node 22 would also satisfy the minimum, but it is already a Maintenance LTS line; prefer Node 24 for the deployment workflow unless a compatibility issue appears.

Keep docs-site dependency management on npm for now. The site has `site/package-lock.json`, CI uses `npm ci`, and the local `just site-build` shortcut mirrors that lockfile install before running `npm run build`. Astro documents npm as a first-class install path. Bun is appropriate for the Pi extension checks in this repository, but standardizing the docs site on Bun would require an intentional lockfile/tooling migration (`bun.lock`, `oven-sh/setup-bun`, and package-manager metadata) without solving Astro's Node runtime requirement. Revisit Bun only if the project decides to migrate all JavaScript package management together.

Upstream references:

- Astro install docs: <https://docs.astro.build/en/install-and-setup/>
- Node.js release policy: <https://nodejs.org/en/about/previous-releases>
- Bun GitHub Actions docs, if a future migration is chosen: <https://bun.com/docs/guides/runtime/cicd>

## Install dependencies

From the repository root with Node.js 24 active (see `site/.node-version`):

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
just site-build
```

The shortcut runs `npm ci` to mirror the GitHub Pages workflow, then the `prebuild` hook runs `npm run sync:docs`, and Astro writes static output to `site/dist/`.

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
## GitHub Pages deployment

The workflow `.github/workflows/docs.yml` builds the Starlight site and deploys `site/dist/` to GitHub Pages on pushes to `main` or manual dispatch. Pull requests run the build and upload step but skip deployment.

Repository setup required in GitHub:

1. Open **Settings → Pages**.
2. Set **Build and deployment → Source** to **GitHub Actions**.
3. Ensure Actions are enabled for the private repository and the `github-pages` environment can deploy.

The site config currently uses `site: 'https://algorant.github.io'` and `base: '/tandem'`, matching the expected GitHub Pages project URL.
