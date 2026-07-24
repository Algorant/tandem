# Tandem v0.6.3

Tandem v0.6.3 simplifies delegation claims and adds supported correction paths for decision records.

## Protocol and CLI

- Accords now begin directly with `tandem accord claim`; the redundant `ready` action and new-status surface are removed.
- Existing legacy `ready` accord documents remain readable, while new work uses direct claim for ordinary delegation and recovery.
- Added `tandem decision update` to amend a decision’s title, body, or ADR status without manual Markdown editing.
- Added `tandem decision withdraw <id> --reason <text>` for decisions created in error. Withdrawal preserves the record with timestamped reason metadata and an audit event.

## TUI and Pi integration

- Decision-view guidance now points to supported decision update/withdraw workflows.
- Pi-Tandem’s typed accord action surface and smoke workflow now use direct claim rather than ready-then-claim ceremony.
