---
id: task-95
uid: e62d583b-ccae-4978-a5be-6247194e0731
type: task
kind: "papercut"
title: "rules add returns provisional ID without reporting final rename"
state: todo
priority: "low"
tags: ["rules"]
accord:
  status: "ready"
  acceptance: ["A successful synced `tandem rules add --json` returns the final rule ID, or reports the provisional-to-final mapping in `sync.renamed`."]
  validation: ["$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  updatedAt: "2026-11-08T02:59:38Z"
createdAt: "2026-10-29T03:16:00Z"
updatedAt: "2026-11-08T02:59:38Z"
effort: "small"
relatedFiles: ["tandem/src/cli/commands.rs"]
---
On 0.17.3, `rules add --json` with category prefer returned `prefer-new-ff7c64e3` and sync data reported `status: synced` with an empty `renamed` map, yet `rules list` immediately showed the rule as `prefer-7`.

## Staleness check / narrowed fix
Since filing, 0.17.4 changed only TUI Board filters, not the CLI rules response. `cli/commands.rs` still maps rule IDs solely through its own sync report; concurrent task creation already solved the equivalent problem in 97d3e9d (task-63). Reuse existing `app::rules::current_rule_id` for successful synced rules-add responses and preserve saved-success + known ID with a warning on resolver error; no new registry, resolver or protocol changes. Verify concurrent native rules adds on a disposable local Git/bare-remote board, and ensure every synced response ID resolves to its own numbered rule. No mocks, no maintained tests, no live-board repro writes, no release or push.