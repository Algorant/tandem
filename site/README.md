# Tandem docs site

Astro Starlight project for rendering Tandem documentation.

Canonical Markdown source lives in `../docs/`. The site project owns rendering, navigation, and static build tooling. Generated Markdown under `src/content/docs/` is produced by the sync script and is not the source of truth.

## Local workflow

```sh
npm install
npm run dev
npm run build
```

Useful scripts:

- `npm run sync:docs` — copy canonical Markdown from `../docs/` into Starlight's content collection.
- `npm run dev` — sync docs, then start Astro's local dev server.
- `npm run build` — sync docs, then write static output to `dist/`.
- `npm run preview` — preview the built `dist/` output locally.

Do not commit `node_modules/`, `.astro/`, `dist/`, or generated Markdown copies under `src/content/docs/`.
## GitHub Pages deployment

The workflow `.github/workflows/docs.yml` builds the Starlight site and deploys `site/dist/` to GitHub Pages on pushes to `main` or manual dispatch. Pull requests run the build and upload step but skip deployment.

Repository setup required in GitHub:

1. Open **Settings → Pages**.
2. Set **Build and deployment → Source** to **GitHub Actions**.
3. Ensure Actions are enabled for the private repository and the `github-pages` environment can deploy.

The site config currently uses `site: 'https://algorant.github.io'` and `base: '/tandem'`, matching the expected GitHub Pages project URL.
