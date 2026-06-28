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
