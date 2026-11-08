---
id: task-95
uid: e62d583b-ccae-4978-a5be-6247194e0731
type: task
kind: "papercut"
title: "rules add returns provisional ID without reporting final rename"
state: "in-progress"
priority: "low"
tags: ["rules"]
accord:
  status: "delivered"
  acceptance: ["A successful synced `tandem rules add --json` returns the final rule ID, or reports the provisional-to-final mapping in `sync.renamed`."]
  claimedAt: "2026-11-08T02:59:40Z"
  deliveredAt: "2026-11-08T03:01:38Z"
  validation: ["$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  summary: "Synced rules-add responses now resolve provisional IDs with current_rule_id and preserve the known ID with a warning if resolution fails, without changing edit/delete behavior."
  evidence: ["A successful synced `tandem rules add --json` returns the final rule ID, or reports the provisional-to-final mapping in `sync.renamed`.: Ran concurrent native adds in separate clones of a temporary Git repo backed by a local bare remote: the synced response returned `prefer-2`, which `rules list` in that clone showed for its own `beta` rule; the competing add correctly reported pending after a ref-lock race and retained its provisional ID. Temporary repository was removed."]
  filesChanged: ["tandem/src/cli/commands.rs"]
  updatedAt: "2026-11-08T03:01:38Z"
createdAt: "2026-10-29T03:16:00Z"
updatedAt: "2026-11-08T03:01:38Z"
effort: "small"
relatedFiles: ["tandem/src/cli/commands.rs"]
assignee: "published-rule-response"
---
On 0.17.3, `rules add --json` with category prefer returned `prefer-new-ff7c64e3` and sync data reported `status: synced` with an empty `renamed` map, yet `rules list` immediately showed the rule as `prefer-7`.

## Staleness check / narrowed fix
Since filing, 0.17.4 changed only TUI Board filters, not the CLI rules response. `cli/commands.rs` still maps rule IDs solely through its own sync report; concurrent task creation already solved the equivalent problem in 97d3e9d (task-63). Reuse existing `app::rules::current_rule_id` for successful synced rules-add responses and preserve saved-success + known ID with a warning on resolver error; no new registry, resolver or protocol changes. Verify concurrent native rules adds on a disposable local Git/bare-remote board, and ensure every synced response ID resolves to its own numbered rule. No mocks, no maintained tests, no live-board repro writes, no release or push.