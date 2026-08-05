# Site overhaul papercuts

Non-blocking issues found during the combined docs-site validation. Record these separately so the page overhaul can continue.

## Broken Extensions source link

- **Found in:** `docs/extensions/index.md`
- **Check:** `cd site && bun run check:links`
- **Error:** `/extensions/` links to `../../extensions/pi-tandem/`, which resolves outside the static site output.
- **Proposed fix:** Link to the published GitHub source or use a valid site URL when the extension documentation is published.

## Broken Quickstart TUI fragment

- **Found in:** `docs/quick-start/index.md`
- **Check:** `cd site && bun run check:links`
- **Error:** `/quick-start/` links to `/tui/#views`, but the TUI page heading is currently `Views and navigation`, so the `views` fragment does not exist.
- **Proposed fix:** Update the link to an existing TUI fragment, or add a stable `Views` heading/anchor.
