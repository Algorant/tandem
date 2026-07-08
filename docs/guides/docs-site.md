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

Use Bun for docs-site dependency management and script execution. The site has `site/bun.lock`; use `bun install --frozen-lockfile` when validating the committed lockfile. Keep `site/package.json` package-manager metadata aligned with the Bun version used to generate the lockfile. Bun is the default package manager per decision-2. Preserve an npm fallback only as a documented exception that records what Bun avenues were tried, why they failed, and what condition would allow revisiting the exception. The GitHub Pages workflow and `just` shortcuts install from `bun.lock`, run docs scripts with Bun, and keep the build entrypoint in CI before link validation.

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

The shortcut mirrors the GitHub Pages workflow's install and build phase with `bun install --frozen-lockfile`, then `bun run build`; the `prebuild` hook runs `bun run sync:docs`, and Astro writes static output to `site/dist/`.

For the full local quality gate from inside `site/`, run:

```sh
bun run check:docs
```

`check:docs` builds the site and then runs the internal link checker against `site/dist/`.

## Link check built output

```sh
cd site
bun run build
bun run check:links
```

`check:links` validates local links, assets, and fragments in built HTML. It intentionally skips external URLs so docs CI does not fail because of transient remote outages or bot-blocking. Use manual browser checks or an external URL checker when changing outbound links that need extra scrutiny.

## Manual sync

```sh
cd site
bun run sync:docs
```

Use this when you want to inspect the generated Starlight content before previewing or building.

## Docs update checklist

1. Edit canonical Markdown under `docs/`; never edit generated copies in `site/src/content/docs/`.
2. Update `site/astro.config.mjs` navigation when pages are added, removed, renamed, or moved.
3. Use `cd site && bun run dev` for local preview. The `predev` hook syncs source docs first.
4. Use `cd site && bun run sync:docs` only when you need to inspect the generated Starlight content collection without starting Astro.
5. Before review, run `cd site && bun run check:docs`. If dependencies changed, run `bun install --frozen-lockfile` first.
6. Check `git status --short --ignored site/src/content/docs`: copied Markdown should be ignored, and only `.gitignore` plus `README.txt` should be tracked in that generated tree.

## Maintenance notes

- Commit source changes under `docs/` and site tooling changes under `site/`.
- Do not commit `site/dist/`, `site/.astro/`, or `site/node_modules/`.
- Keep generated `site/src/content/docs/**/*.md` out of version control; it exists only to bridge canonical docs into Starlight. The nested `.gitignore` ignores copied Markdown while allowing the tracked `.gitignore` and `README.txt` notice.

## GitHub Pages deployment

The workflow `.github/workflows/docs.yml` builds the Starlight site with Node from `site/.node-version`, installs Bun with `oven-sh/setup-bun`, runs `bun install --frozen-lockfile`, runs `bun run build`, checks built internal links with `bun run check:links`, and deploys `site/dist/` to GitHub Pages on pushes to `main` or manual dispatch. Pull requests run the build, link check, and upload step but skip deployment.

Production docs URL: <https://trytandem.dev/>. The Astro config uses `site: 'https://trytandem.dev'` and `base: '/'` so generated links and assets target the custom-domain root. The GitHub project Pages URL remains <https://algorant.github.io/tandem/> and should redirect to the custom domain once GitHub Pages accepts the domain.

Repository setup required in GitHub:

1. Open **Settings → Pages**.
2. Set **Build and deployment → Source** to **GitHub Actions**.
3. Set **Custom domain** to `trytandem.dev`.
4. After DNS resolves and GitHub provisions the certificate, enable **Enforce HTTPS**.
5. Ensure Actions are enabled for the repository and the `github-pages` environment can deploy.

Useful `gh` checks:

```sh
gh api repos/Algorant/tandem/pages \
  --jq '{html_url, cname, https_enforced, build_type, source, https_certificate}'

gh api repos/Algorant/tandem/pages/health
```

If the custom domain needs to be set from the CLI:

```sh
gh api --method PUT repos/Algorant/tandem/pages -f cname=trytandem.dev
# Retry after DNS/certificate provisioning if GitHub reports that no certificate exists yet.
gh api --method PUT repos/Algorant/tandem/pages -F https_enforced=true
```

DNS records for Namecheap **Advanced DNS**:

| Type | Host | Value |
| --- | --- | --- |
| A | `@` | `185.199.108.153` |
| A | `@` | `185.199.109.153` |
| A | `@` | `185.199.110.153` |
| A | `@` | `185.199.111.153` |
| AAAA | `@` | `2606:50c0:8000::153` |
| AAAA | `@` | `2606:50c0:8001::153` |
| AAAA | `@` | `2606:50c0:8002::153` |
| AAAA | `@` | `2606:50c0:8003::153` |
| CNAME | `www` | `algorant.github.io` |

Remove Namecheap parking/default `@` and `www` records before adding the GitHub Pages records, do not add wildcard records, and allow up to 24 hours for DNS and certificate propagation. `site/public/CNAME` records the intended custom domain in the built artifact; for GitHub Actions Pages, the repository Pages setting is still the authority.

### Branded installer redirect

The primary branded installer command is:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
```

`/install.sh` must be maintained as a real HTTP redirect to the cargo-dist generated installer:

```text
https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh
```

GitHub Pages can serve static files and custom 404 pages, but it does not support arbitrary path-level `301`/`302` redirects from repository configuration. Do not restore `site/public/install.sh` as a shell wrapper; that duplicates installer forwarding logic in the docs site and bypasses cargo-dist's release asset selection and checksum behavior.

Preferred provider setup with Cloudflare:

1. Move `trytandem.dev` DNS to Cloudflare or otherwise put Cloudflare in front of the hostname.
2. Keep the apex site proxied to GitHub Pages with the same `A`/`AAAA` records listed above, and keep `www` as a proxied `CNAME` to `algorant.github.io`.
3. Add a **Redirect Rule** matching `Hostname equals trytandem.dev` and `URI Path equals /install.sh`.
4. Set **Static redirect** target URL to `https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh` with status code `302` (or `307`). Preserve query string can be enabled, but the installer command does not require one.
5. Ensure the rule runs before any broader site forwarding rules.

Equivalent hosting options are acceptable if they create the same HTTP redirect without a Tandem-maintained shell shim, for example a Netlify `_redirects` entry or Vercel `redirects` rule if the docs site is moved to that provider.

Redirect verification after provider configuration:

```sh
curl -I https://trytandem.dev/install.sh
curl -fsSL https://trytandem.dev/install.sh | head
```

The first command should report a `30x` response with `location: https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh`.

Launch verification:

```sh
dig trytandem.dev +noall +answer -t A
dig trytandem.dev +noall +answer -t AAAA
dig www.trytandem.dev +noall +answer -t CNAME
curl -I https://trytandem.dev/
curl -I https://www.trytandem.dev/
curl -I https://algorant.github.io/tandem/
```
