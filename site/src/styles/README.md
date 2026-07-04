# Docs theme assets

This directory is the vendorable boundary for the Tandem docs-site theme.

- `verdigris.css` is the standalone Starlight `customCss` theme. It maps the Tandem Verdigris palette to Starlight CSS roles and contains only site/theme presentation rules, including Markdown-compatible card/tag helpers used by the theme tester page.
- `shiki/verdigris-*.jsonc` are the paired Expressive Code/Shiki syntax themes loaded by `site/astro.config.mjs`.
- `../../docs/guides/theme-tester.md` is the canonical source page for visually validating these assets through the docs sync pipeline.

Keep runtime wiring in `site/astro.config.mjs`. Do not add a theme package, duplicate Starlight internals, or move these assets outside `site/src/styles/` without an explicit packaging/vendoring decision.
