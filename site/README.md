# Tandem docs site

Astro Starlight project for rendering Tandem documentation.

Canonical Markdown source lives in `../docs/`. The site project owns rendering, navigation, and static build tooling. Generated Markdown under `src/content/docs/` is produced by the sync script and is not the source of truth.

## Local workflow

Use Node.js 24 (see `.node-version`) and Bun for local dependency management and scripts.

```sh
bun install
bun run dev       # sync docs and start Astro locally
bun run build     # sync docs and build site/dist/
bun run preview   # preview an existing site/dist/ build
```

The docs-site package state is locked in `bun.lock`; use `bun install --frozen-lockfile` for reproducible validation. If a workflow needs an npm fallback, document it as a Bun exception with the attempts made, the incompatibility encountered, and the condition for revisiting the fallback.

For local docs quality checks, run the build and internal link checker:

```sh
bun run check:docs
```

`check:docs` runs `bun run build` and then validates local links, assets, and fragments in the built HTML. External URLs are intentionally skipped so local and CI checks are deterministic.

## Theme assets

The site uses a standalone vendored Verdigris theme instead of a Starlight theme package. Runtime wiring lives in `astro.config.mjs`; the vendorable asset boundary is `src/styles/`:

- `src/styles/verdigris.css` maps the Tandem Verdigris palette onto Starlight roles and component styling.
- `src/styles/shiki/verdigris-*.jsonc` provides paired Expressive Code/Shiki syntax themes.
- `../docs/guides/theme-tester.md` is the canonical Markdown page for visual theme validation.
- `src/styles/README.md` documents the local theme boundary.

Useful scripts:

- `bun run sync:docs` — clear Starlight's generated content cache, remove generated Markdown copies, then copy canonical Markdown from `../docs/` into Starlight's content collection while preserving `src/content/docs/.gitignore` and `README.txt`.
- `bun run dev` — sync docs, then start Astro's local dev server with `--force` so rendered code blocks pick up theme changes.
- `bun run build` — sync docs, then write static output to `dist/` with `--force`.
- `bun run check:links` — validate internal links, assets, and fragments in an existing `dist/` build.
- `bun run check:docs` — run the full local docs quality gate: build, then link-check.
- `bun run preview` — preview the built `dist/` output locally.

## Docs update checklist

1. Edit canonical source under `../docs/`; do not edit generated Markdown copies under `src/content/docs/`.
2. If you add, remove, or rename top-level pages, update the Starlight sidebar in `astro.config.mjs`.
3. Run `bun run dev` for live preview, or `bun run sync:docs` when you only need to inspect generated Starlight content.
4. Before opening a PR, run `bun install --frozen-lockfile` when dependencies changed, then `bun run check:docs`.
5. Confirm `git status --short --ignored src/content/docs` shows generated Markdown as ignored, not staged.

Do not commit `node_modules/`, `.astro/`, `dist/`, or generated Markdown copies under `src/content/docs/`. The nested `.gitignore` in `src/content/docs/` keeps copied docs ignored while allowing the tracked `.gitignore` and `README.txt` notice.

## GitHub Pages deployment

The workflow `.github/workflows/docs.yml` builds the Starlight site, runs the internal link checker against `site/dist/`, and deploys `site/dist/` to GitHub Pages on pushes to `main` or manual dispatch. Pull requests run the build, link check, and upload step but skip deployment.

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

The intended branded installer command is:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
```

`/install.sh` must be a real HTTP redirect to the cargo-dist generated installer:

```text
https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh
```

GitHub Pages can serve static files and custom 404 pages, but it does not support arbitrary path-level `301`/`302` redirects from repository configuration. Do not restore `site/public/install.sh` as a shell wrapper; that duplicates installer forwarding logic in the docs site. Until external hosting is configured, document the direct GitHub Release installer URL as the available install command.

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
