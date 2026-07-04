---
title: Theme tester
description: A low-noise page for validating the Verdigris docs theme.
---

# Theme tester

This page intentionally exercises the Verdigris docs theme. Use it when changing `site/src/styles/verdigris.css`, the Expressive Code themes, or Starlight navigation styling.

It is not product documentation. It is a visual maintenance page for checking headings, links, inline code, code blocks, callouts, card-like surfaces, badges/tags, lists, and tables in one predictable place.

:::note[Maintenance note]
The canonical docs sync currently copies plain `.md` files from `docs/` into Starlight. This page therefore uses Markdown-compatible syntax and a few raw HTML helpers from `verdigris.css` for cards/tags instead of MDX-only Starlight `<Card>` or `<Badge>` components.
:::

## Heading scale

# H1 verdigris headline

A page-level heading should read green and confident without washing out the body copy. This paragraph includes an [internal docs-site workflow link](./docs-site.md), an [external Astro link](https://astro.build/), and inline code such as `tandem accord deliver`.

## H2 aqua section

H2 headings should carry the cool aqua role and provide a clear section break.

### H3 brass subsection

H3 headings should introduce brass/ochre warmth for implementation detail.

#### H4 moss detail

H4 headings should use the moss accent for supporting structure.

##### H5 cream label

H5 headings are small detail labels with a cream/brass feel.

###### H6 muted note

H6 headings should stay quiet and muted.

## Body copy and links

Tandem docs should remain readable before they become decorative. Body copy uses a warm neutral foreground, while links and inline code carry enough Verdigris identity to be discoverable during scanning.

- Internal link: [Guides overview](./index.md)
- External link: [Starlight authoring content](https://starlight.astro.build/guides/authoring-content/)
- Inline code: `state: validation`, `accord.status: delivered`, and `bun run build`

## Fenced code blocks

The following blocks verify Expressive Code rendering and Verdigris syntax highlighting.

```ts
export type AccordStatus = 'ready' | 'claimed' | 'delivered' | 'accepted' | 'rework';

export function nextAction(status: AccordStatus): string {
  if (status === 'delivered') return 'review visually before acceptance';
  return `continue ${status}`;
}
```

```sh
cd site
bun install --frozen-lockfile
bun run build
```

```toml
theme = "verdigris"
transparent_background = false
badge_style = "muted"

[board.badges.tags.docs]
tone = "success"
```

## Blockquotes

> A good validation page makes broken styling obvious before it reaches release notes. The blockquote should have a restrained Verdigris rail, a soft background, and readable body text.

## Starlight asides

:::tip[Tip]
Tip asides should use the Verdigris/green role and remain calmer than the active navigation state.
:::

:::caution[Caution]
Caution asides should use the brass role without overwhelming nearby paragraphs.
:::

:::danger[Danger]
Danger asides should remain distinct from caution and success while preserving contrast.
:::

## Markdown-compatible cards and tags

<div class="theme-card-grid">
  <article class="theme-card">
    <p class="theme-card__eyebrow">Verdigris</p>
    <h3>Validation path</h3>
    <p>Primary theme surfaces should feel Tandem-specific without becoming neon.</p>
  </article>
  <article class="theme-card theme-card--aqua">
    <p class="theme-card__eyebrow">Aqua</p>
    <h3>Navigation context</h3>
    <p>Aqua accents separate links, secondary headings, and supporting UI.</p>
  </article>
  <article class="theme-card theme-card--brass">
    <p class="theme-card__eyebrow">Brass</p>
    <h3>Implementation detail</h3>
    <p>Brass/ochre adds warmth for warnings, H3s, and metadata surfaces.</p>
  </article>
  <article class="theme-card theme-card--moss">
    <p class="theme-card__eyebrow">Moss</p>
    <h3>Supporting structure</h3>
    <p>Moss keeps lower-level hierarchy visible without competing with H1.</p>
  </article>
</div>

Tag examples: <span class="theme-token theme-token--success">accepted</span> <span class="theme-token theme-token--note">validation</span> <span class="theme-token theme-token--caution">manual review</span> <span class="theme-token theme-token--danger">blocked</span> <span class="theme-token theme-token--muted">archived</span>

## Lists and tables

1. Check dark and light mode.
2. Check sidebar current-page state.
3. Check code fences before accepting theme work.

| Surface | Expected accent | Maintenance note |
| --- | --- | --- |
| H1 | Verdigris green | Primary page identity |
| H2 | Aqua | Section separation |
| H3 | Brass/ochre | Warm implementation detail |
| H4 | Moss | Supporting hierarchy |
| Code | Verdigris syntax | Must render as Expressive Code |
