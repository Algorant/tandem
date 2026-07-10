# Tandem v0.4.3

Tandem v0.4.3 is a maintenance release focused on TUI rendering, documentation clarity, and reliable AUR publishing.

## Highlights

- Fixed Epic Board child-row clipping so titles and metadata render correctly within the available width.
- Simplified the documentation site into a smaller, more useful information architecture.
- Rewrote the project README to reflect Tandem's current purpose, installation paths, repository layout, and workflow.
- Stabilized the automated `tandem-bin` AUR publishing workflow.

## TUI

- Reserved space for the Board selection marker when calculating Epic Board row width.
- Improved row-width accounting for indentation, badges, titles, spacing, and right-aligned metadata.
- Prevented child titles and metadata from overflowing or clipping in narrower Epic Board layouts.

## Documentation

- Reduced the docs landing page to a minimal Home.
- Simplified the sidebar around Home, Quickstart, Overview, Workflows, and Integrations.
- Removed duplicate page headings already supplied by Starlight.
- Added the Skills documentation placeholder.
- Updated release, installer, and AUR maintenance guidance.
- Replaced the planning-oriented root README with a current project overview.

## Packaging

- Fixed parsing of cargo-dist checksum entries that use binary-file markers.
- Normalized multiline AUR SSH private-key secrets.
- Made AUR SSH identity and host-key handling more reliable.
- Preserved Git metadata ownership while generating and committing package files.
- Added writable `makepkg` build, source, and package output directories.
- Improved generation and publication of `PKGBUILD` and `.SRCINFO`.
