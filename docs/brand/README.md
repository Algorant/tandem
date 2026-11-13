# tandem wordmark — reference artwork

The current tandem identity is the interlaced-strands mark used on
[trytandem.dev](https://trytandem.dev/) and in this repository's README. The four
numbered exploration boards below are **historical**: they were drawn during the
November 2026 wordmark reconciliation (task-110) and none of them was adopted.

## Active source

The active artwork lives in `site/src/assets/` and is what the docs header, the
README banner, the favicon and the social preview are built from:

- `site/src/assets/tandem-mark.svg` — mark for dark surfaces
- `site/src/assets/tandem-mark-light.svg` — mark for light surfaces

The two SVG copies in this folder are reference-only and byte-identical to those
files. Nothing here is generated: every file is a plain, hand-kept asset, and no
brand-generator script exists in this repository.

The mark is one geometry: both variants carry the same paths, mask and stroke
width, and differ only in the colour of the ink strand (`#ebdbb2` dark,
`#504945` light). The favicon and the README/site banners embed that same path
data — the favicon at a heavier stroke weight for a 64 px tile — rather than a
redrawn copy.

The name `tandem` is not part of the mark. On the live site it is separate HTML
text: the theme's system-sans stack at 24 px / 600 with 0.36 px tracking, in the
accent colour (`#d5e7d0` dark, `#427b58` light). The banners keep it as SVG text
in that same stack, so it resolves to the same face the site uses on the same
machine.

## Files

| file | what it is |
| --- | --- |
| `tandem-mark.svg` | Reference copy of the active dark-surface mark (`site/src/assets/tandem-mark.svg`). |
| `tandem-mark-light.svg` | Reference copy of the active light-surface mark (`site/src/assets/tandem-mark-light.svg`). |
| `current-lockup-dark.png` | High-resolution dark render of the current docs-site lockup, as shown to Algorant in task-110; unchanged. |
| `current-lockup-light.png` | The same render for the light theme; unchanged. |
| `comparison-current-marks.png` | Task-110 comparison board: the old repo cassette banner and the docs-site header lockup side by side, with the current banner and header reconstructions. |
| `exploration-1-twin-t.png` | Historical exploration board 1, "twin t": two stems sharing one crossbar. Not adopted. |
| `exploration-2-weave.png` | Historical exploration board 2, "weave": two paths cross once and hold. Not adopted. |
| `exploration-3-embrace.png` | Historical exploration board 3, "embrace": two arcs around one shared core. Not adopted. |
| `exploration-4-yoke.png` | Historical exploration board 4, "yoke": two strands arrive, one continues. Not adopted. |

## Notes on the historical material

- The comparison board and the four exploration boards are the original task-110
  renders, kept as-is. They show the retired cassette banner and the four
  unadopted marks on purpose: this folder is the record of that exploration, not
  an active asset set.
- The cassette banner that used to lead the README and the docs landing page is
  gone from every active surface; it survives only inside the comparison board.
- The exploration boards set the wordmark in Adwaita Sans as a stand-in face.
  The current lockup renders use the Arial-metric face a Linux browser actually
  resolves for the site's stack. Neither is a typeface decision for the brand.
- The pre-adoption cards are reference images only. The shipped banners are the
  SVGs in `site/public/banner-dark.svg` and `site/public/banner-light.svg`, used
  by the README `<picture>` and by the docs landing page.
