---
id: task-53
uid: 26e2bc8c-edb8-46d2-98d8-00e628cd44a0
type: task
title: "Rules end to end: starter set, categories, toggles and agent visibility — mockup first"
state: "validation"
priority: "medium"
tags: ["rules", "ui"]
accord:
  status: "delivered"
  acceptance: ["Phase one: an Astra or Fable Worker publishes an explicitly proposed Sideshow end-to-end rules mockup using a toy project, showing the seeded set, always/never/prefer/context categories, on/off behavior and what agents see, with Excalidraw-method diagram assets; Algorant reviews it before any implementation.", "After separate approval: normative on/off representation/defaults, CLI/JSON and context refresh behavior are decided before code; disabled rules remain discoverable and retain IDs, category, text/source and supported metadata.", "After separate approval: init seeds agreed categorized rules, reports them, and idempotently adds a short rules pointer to existing AGENTS.md/CLAUDE.md without creating absent files.", "After separate approval: TUI keyboard/mouse state and agent-facing enabled-rule selection are unambiguous, reload-safe and not color-only; any external integration change is separately authorized.", "After separate approval: temporary native checks cover seeded/default/present-absent/idempotent pointer cases, toggling/readback and context refresh; published guidance matches the verified contract."]
  claimedAt: "2026-10-17T02:47:10Z"
  deliveredAt: "2026-10-20T23:10:51Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Prototype and implementation plan approved by Algorant via Herdra; same retained Astra Worker is approved through implementation.", "Implement normative semantics first, shared app/project operations next; CLI/TUI peers consume them. No external adapter implementation or new tool/framework.", "Temporary feature tests/fixtures only; remove before delivery. Existing tests may receive mechanical field/expectation updates.", "Actual Rules TUI screenshot showing On/Off, counts and toggle goes to Herdra for Algorant BEFORE release.", "Worker owns scoped implementation/clean commits only. Parent owns source reconciliation/integration/push, task lifecycle, preview routing, release, machine upgrades, TUI restarts and worktree cleanup."]
  summary: "Implemented accepted rules prototype as whole Task: six categorized enabled starters, safe existing-agent-file pointers, enabled-only guidance and retained on/off management via shared CLI/TUI operations; clean source47ca4cf. Parent verified native checks, byte/metadata/pointer matrix, real keyboard+mouse ANSI captures and enabled-only web. Actual UI review remains the pre-release gate."
  evidence: ["Algorant accepted phase-one four-post proposal before implementation; Herdra approved build plan/current retained model and exact symlink/web boundaries. Native EA sources/PNGs retained; no external adapter implementation.", "Parent independently passed 13 isolated checks: seed/pointer none/single/both/shared-symlink/exact/dangling, pre-init nonregular refusal, omitted=true/no rewrite, no-op bytes/events, strict bool errors, CRLF/nested metadata/body retention, disabled-state text edit and same-ID re-enable. Fixtures/harness removed.", "Parent final-source justdevcheck/fmt/strictclippy/diffcheck passed. Worker Git/non-Git matrix covered published/provisional IDs, two-clone refresh, pointer target race/mode, task86/task87 regressions and removed all new temporary native checks.", "Real release-profile TUI at47ca4cf: Space changed six stored5on1off→6on0off; actual SGR mouse action restored5on1off with same selectedalways-2. Category title/width defects fixed and recaptured. Native ANSI files and PNGs retained under/tmp/task53-tui-*; no invented labels/counts.", "WebAPI and Obscura page showed exactly five enabled records, disabledalways-2 absent; assignment schema untouched. Browser/server/ownedpreview panes stopped. Screenshot set at http://desktop.tail1cefc.ts.net:8228/session/XLDMECmdbMI (postPGtIuqGQnPs), boundedHTTP200. Await Herdra/Algorant actualUI review before integration/release. Shared safe justdev route retained for review."]
  filesChanged: ["AGENTS.md", "docs/agents/index.md", "docs/cli/index.md", "docs/quick-start/index.md", "docs/tui/index.md", "plan/agent-adapter-implementation-handoffs.md", "protocol/README.md", "tandem/src/app/project.rs", "tandem/src/app/rules.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "tandem/src/project/agent_files.rs", "tandem/src/project/frontmatter.rs", "tandem/src/project/mod.rs", "tandem/src/project/rules.rs", "tandem/src/project/rules_contract_tests.rs", "tandem/src/project/sync.rs", "tandem/src/protocol/config.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/rules.rs", "tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/tests/init_behavior.rs", "tandem/tests/sync_behavior.rs"]
  updatedAt: "2026-10-20T23:10:51Z"
createdAt: "2026-10-01T18:27:09Z"
updatedAt: "2026-10-20T23:10:51Z"
effort: "large"
references: ["task-54", "task-20", "task-81"]
relatedFiles: ["protocol/README.md", "tandem/src/protocol/config.rs", "tandem/src/project/rules.rs", "tandem/src/app/rules.rs", "tandem/src/app/project.rs", "tandem/src/cli/model.rs", "tandem/src/cli/commands.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/reload.rs", "docs/agents/index.md", "docs/cli/index.md", "docs/tui/index.md"]
assignee: "worker-task-53-b81689dc"
validation.criterion: "Phase one: an Astra or Fable Worker publishes an explicitly proposed Sideshow end-to-end rules mockup using a toy project, showing the seeded set, always/never/prefer/context categories, on/off behavior and what agents see, with Excalidraw-method diagram assets; Algorant reviews it before any implementation."
validation.note: "Phase-one prototype judgment ONLY: internalmilestone53-1 delivered (Subtasks cannot separately enterreview). Parent verified actualfourposts matchpayload, bothnativePNGpixels/hashes andeditablesourcespresent, gitclean,HTTP200. http://desktop.tail1cefc.ts.net:8228/session/XLDMECmdbMI. Futureimplementationcriteria remain gated/unimplemented. DoNOTcomplete/worker_finishroot; awaitprototypeacceptance andnewplan. EA slotreleased."
validation.requestedAt: "2026-10-17T03:58:16Z"
validation.reviewer: "Algorant"
---
## Approved scope

Algorant via Herdra accepted prototype milestone task-53-1 and its four-post Sideshow proposal (http://desktop.tail1cefc.ts.net:8228/session/XLDMECmdbMI), then approved implementation steps 1–7 in m084bd177. Keep the exact retained Astra Worker through implementation. Root consolidates canceled tasks54/20; prototype acceptance alone did not complete it. No more EA/vault work needed; editable sources and native PNGs remain retained.

## Rules contract

Normative protocol first. Optional boolean enabled, omission=true without rewrite/migration; new rules/seeds explicit true. Disabled records remain discoverable with same UID/ID/category/text/source/createdAt/unknown metadata. Real transition patches enabled+updatedAt and emits existing rules.updated event; already-matching state is a no-op (no write/event/timestamp churn). No disable warning/confirmation. Ordinary edits preserve enabled and unrelated metadata. Keep protocol0.5.0; older readers ignore disabled state, requiring coordinated0.17.3 upgrades before relying on it.

Shared app/project operations, peer CLI/TUI. rules list defaults enabled-only; --all includes disabled; rules enable ID / disable ID. Existing {ok,data,warnings}, list data remains array with enabled; state result finalid/enabled/changed/sync. Preserve current category filtering and publication/UID/provisional-ID behavior. No silent failed-read fallback. TUI management retains ALL rules; literal On/Off, counts, Space and clickable selected-state control; keep Enter preview/navigation and identity-based selection through reload/restart.

## Accepted fresh-init starter set and pointer

always: Read current project rules before starting work. / Use Tandem CLI/tools for coordination changes.
never: Do not edit board files directly. / Do not alter archived records.
prefer: Keep work small and independently verifiable.
context: The Board holds active work; Logs hold completed history.

Seed through shared creation/allocation/event paths and report final published identities plus per-agent-file pointer outcomes. Exact line: Read the current project rules with `tandem rules list` before starting work.

Only existing AGENTS.md/CLAUDE.md at resolved project root; both if present. Exact whole-line match skips insertion. Preserve prose/bytes/newlines/mode; no absent-file creation or init-rerun feature. Herdra-approved safety policy m6d5e8165: symlinks to regular files are supported by appending IN PLACE to resolved target (open append/no create, never replace link or mode). If both names resolve to same file, pointer inserted once. Missing targets skipped; directories/non-regular targets explicit error. Detect change/disappearance. Preflight before board init; late failure reports partial init precisely, no rollback framework.

## Context, verification and gates

Session-start agents re-read enabled rules via CLI, check success/warnings and apply guidance. Fresh-local reads use normal remote-refresh/warning contract, not a new guaranteed-remote service. No push into running agents; failed reads reported/resolved, never silently old/empty guidance. Update Tandem-owned docs/handoff guidance only; external adapter edits require separate authorization.

Temporary isolated Git/non-Git matrix: legacy omission/strict bool, toggle/no-op/metadata/text-edit retention, seeds/pointer present/absent/both/idempotence/symlink-alias/bytes/CRLF/mode/error cases, publication/IDs, context read selection, TUI mouse+keyboard/reload/restart/counts, and task86/task87 regressions. Native dev-check/fmt/strictclippy plus applicable docs build. Remove temporary tests/fixtures before full-root READY report.

Parent reconciled/pushed main to21be411 (task86 rebased onto task87/0.17.2); parent updates clean idle Worker to it before code. Worker re-inspects baseline and flags material deltas first. Parent verifies actual ANSI/real TUI screenshot and sends to Herdra for Algorant BEFORE release. Shared safe justdev preview route; clean during cleanup. Parent owns integration/task acceptance,0.17.3 Cargo/lock+curated RELEASES section, justrelease/installer checks, coordinated desktop+archbox upgrades and existing-TUI restarts; x1nano when online. No Worker push/release/machine changes.