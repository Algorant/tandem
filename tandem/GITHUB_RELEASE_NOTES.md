# Tandem v0.6.1

Tandem v0.6.1 adds safe Task body editing and auditable cancellation before the larger Rust architecture refactor begins.

## Highlights

- `tandem update <id> --body <markdown>` now replaces or clears an active Task's complete Markdown body while preserving unrelated frontmatter and unknown fields.
- Body updates are exact and no-op aware: unchanged content does not rewrite the file, timestamp, or event history.
- `tandem cancel <id> --reason <text>` archives mistaken or abandoned Tasks to Logs with a distinct `canceled` outcome while preserving their body, metadata, references, and ID.
- CLI, JSON, TUI Logs, Board rollups, and `pi-tandem` distinguish canceled work from successful completion.

## Bug fixes

- Fixed agents needing to edit `.tandem` Markdown directly to correct an existing Task body.
- Fixed empty, whitespace-only, multiline, Unicode, and leading-dash body replacements being unavailable through `tandem update` and `pi-tandem`.
- Fixed mistaken or intentionally abandoned Tasks having to be recorded as successfully completed to leave the active Board.
- Fixed canceled work being counted as successful completion in hierarchy rollups or rendered as malformed/completed Log history.

## Compatibility note

Existing Logs without `completion.outcome` continue to mean `completed`. Canceled Logs use the additive `completion.outcome: canceled` value while retaining the existing protocol `0.1.0` format.
