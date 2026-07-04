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

Use Bun for docs-site dependency management and script execution. The site has `site/bun.lock`; use `bun install --frozen-lockfile` when validating the committed lockfile. Keep `site/package.json` package-manager metadata aligned with the Bun version used to generate the lockfile. Bun is the default package manager per decision-2. Preserve an npm fallback only as a documented exception that records what Bun avenues were tried, why they failed, and what condition would allow revisiting the exception. The GitHub Pages workflow and `just` shortcuts install from `bun.lock` and run docs scripts with Bun.

Upstream references:

- Astro install docs: <https://docs.astro.build/en/install-and-setup/>
- Node.js release policy: <https://nodejs.org/en/about/previous-releases>
- Bun package manager install docs: <https://bun.com/docs/pm/cli/install>

## Standalone Verdigris theme

The docs site uses a standalone vendored Verdigris theme rather than a Starlight theme package. This keeps Astro 7/Starlight 0.41 compatibility, avoids peer-dependency drift from third-party theme packages, and makes the theme easy to review or vendor elsewhere.

Theme ownership is intentionally scoped to the site project:

- `site/astro.config.mjs` wires Starlight `customCss` to `site/src/styles/verdigris.css` and loads paired Expressive Code themes with `ExpressiveCodeTheme.fromJSONString(...)`.
- `site/src/styles/verdigris.css` defines the Verdigris palette, maps it to Starlight color roles, and styles headings, links, badges/tags, cards, callouts, sidebars, pagination, inline code, and Expressive Code frames.
- `site/src/styles/shiki/verdigris-dark.jsonc` and `site/src/styles/shiki/verdigris-light.jsonc` provide syntax highlighting themes whose UI chrome is synchronized with Starlight via `useStarlightUiThemeColors: true`.
- Expressive Code external stylesheet emission is disabled in the Starlight config so generated code blocks remain self-contained and do not point at stale hashed `ec.*.css` assets after theme changes.
- `site/scripts/sync-docs.mjs` clears Astro's generated `.astro/` content cache before syncing Markdown, and the dev/build scripts pass Astro `--force`, so code-fence HTML is regenerated when theme or Expressive Code settings change.
- `docs/guides/theme-tester.md` is the canonical visual maintenance page for exercising headings, links, code blocks, asides, cards, tags, lists, and tables.
- `site/src/styles/README.md` is the vendoring boundary note for future packaging work.

Do not add `starlight-theme-gruvbox`, downgrade Astro/Starlight, or introduce another theme package unless that trade-off is explicitly approved. If the Verdigris theme is later published as a reusable package, keep this local CSS/Shiki asset boundary as the source to extract from and preserve any third-party license notices for newly vendored assets.

Relevant theme references:

- Starlight CSS customization: <https://starlight.astro.build/guides/css-and-tailwind/>
- Expressive Code themes: <https://expressive-code.com/guides/themes/>
- Starlight themes catalog: <https://starlight.astro.build/resources/themes/>

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

The `predev` hook syncs `../docs/` into Starlight before the dev server starts. The sync script and Astro `--force` flag clear generated content caches so code blocks pick up theme changes.

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
