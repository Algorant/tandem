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

Use Bun for docs-site dependency management and script execution. The site has `site/bun.lock`; use `bun install --frozen-lockfile` when validating the committed lockfile. Keep `site/package.json` package-manager metadata aligned with the Bun version used to generate the lockfile. Bun is the default package manager per decision-2; preserve an npm fallback only if a concrete Bun incompatibility is validated and documented. The GitHub Pages workflow and `just` shortcuts install from `bun.lock` and run docs scripts with Bun.

Upstream references:

- Astro install docs: <https://docs.astro.build/en/install-and-setup/>
- Node.js release policy: <https://nodejs.org/en/about/previous-releases>
- Bun package manager install docs: <https://bun.com/docs/pm/cli/install>

## Gruvbox theme workaround

The Starlight themes catalog lists Starlight Gruvbox as a community theme, and its install guide recommends installing `starlight-theme-gruvbox` and adding `plugins: [gruvbox()]` to the Starlight config. As of `starlight-theme-gruvbox@2.0.0`, the package itself is not compatible with this docs site's dependency stack: it declares peer dependencies on Astro `^6.0.0` and Starlight `^0.38.0`, while this site intentionally uses Astro `^7.0.3` and Starlight `^0.41.1`.

To keep Astro 7 and Starlight 0.41, the site vendors only the theme assets that are compatible with current Starlight APIs:

- `site/src/styles/gruvbox.css` sets Starlight CSS variables via `customCss`.
- `site/src/styles/shiki/gruvbox-*-medium.jsonc` provides the Gruvbox Expressive Code themes via `ExpressiveCodeTheme.fromJSONString(...)`.
- `site/src/styles/shiki/starlight-theme-gruvbox.LICENSE` preserves the upstream MIT license for the adapted assets.

Do not add the incompatible `starlight-theme-gruvbox` package or downgrade Astro/Starlight unless that trade-off is explicitly approved. If the upstream theme publishes Astro 7-compatible peer ranges later, this vendored workaround can be replaced with the package integration.

Relevant theme references:

- Starlight themes catalog: <https://starlight.astro.build/resources/themes/>
- Starlight Gruvbox install guide: <https://starlight-theme-gruvbox.otterlord.dev/guides/install/>
- Theme package/repository: <https://github.com/TheOtterlord/starlight-theme-gruvbox>

## Install dependencies

From the repository root with Node.js 24 active (see `site/.node-version`):

```sh
cd site
bun install
```

## Preview locally

```sh
cd site
bun run dev
```

The `predev` hook syncs `../docs/` into Starlight before the dev server starts.

## Build static output

```sh
just site-build
```

The shortcut mirrors the GitHub Pages workflow with `bun install --frozen-lockfile`, then `bun run build`; the `prebuild` hook runs `bun run sync:docs`, and Astro writes static output to `site/dist/`.

## Manual sync

```sh
cd site
bun run sync:docs
```

Use this when you want to inspect the generated Starlight content before previewing or building.

## Maintenance notes

- Commit source changes under `docs/` and site tooling changes under `site/`.
- Do not commit `site/dist/`, `site/.astro/`, or `site/node_modules/`.
- Keep generated `site/src/content/docs/**/*.md` out of version control; it exists only to bridge canonical docs into Starlight.
## GitHub Pages deployment

The workflow `.github/workflows/docs.yml` builds the Starlight site with Node from `site/.node-version`, installs Bun with `oven-sh/setup-bun`, runs `bun install --frozen-lockfile`, and deploys `site/dist/` to GitHub Pages on pushes to `main` or manual dispatch. Pull requests run the build and upload step but skip deployment.

Repository setup required in GitHub:

1. Open **Settings → Pages**.
2. Set **Build and deployment → Source** to **GitHub Actions**.
3. Ensure Actions are enabled for the private repository and the `github-pages` environment can deploy.

The site config currently uses `site: 'https://algorant.github.io'` and `base: '/tandem'`, matching the expected GitHub Pages project URL.
