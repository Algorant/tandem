# Agent and adapter guidance disposition and implementation handoffs

Status: implementation handoff
Date: 2026-08-03

This document records the research disposition for framework-neutral agent guidance. It is not runtime prompt text. Normative universal semantics live in `protocol/plan/spec.md`; public operational guidance lives in `docs/guides/agents-and-adapters.md`. Active `.tandem` rules remain repository policy.

## Disposition

Existing adapter text was research input, not an authority. The table classifies the behavior by its correct owner.

| Concern found in protocol, CLI, Rules, docs, or adapter material | Classification | Disposition and authority |
| --- | --- | --- |
| Discover the nearest workspace from the intended directory; ask before initialization; never upgrade implicitly | Universal Tandem semantics | Keep in protocol and generic guidance. Tandem discovery and compatibility checks are authoritative. |
| Read all rule categories before planning a mutation | Universal Tandem semantics | Keep in protocol and generic guidance. |
| `always`, `never`, `prefer`, and `context` behavior | Universal Tandem semantics | Define normatively in the protocol. Preserve the stored category without wording-based reclassification. |
| Prohibitions such as “do not modify adapter code” | Universal classification method, repository-specific content | Store the prohibition under `never` in the relevant workspace. Split a separate required handoff into `always` when independently enforceable. |
| Keep workflow state, accord status, and review status separate | Universal Tandem semantics | Keep in protocol and generic guidance. |
| Treat validation output as evidence, not acceptance or lifecycle authority | Universal authority boundary | Keep in generic guidance. Exact actor authority still comes from the assignment, caller, and workspace policy. |
| Use Board, Decisions, Rules, Logs, references, blockers, and events for their defined purposes | Universal Tandem semantics | Keep in protocol and generic guidance. |
| Let Tandem allocate IDs, classify relationships, validate structure, mutate Markdown, and manage actor identity | Universal adapter boundary | Keep in protocol and generic guidance. Adapters pass through Tandem results. |
| Use a non-interpolating process API and structured read output | Universal adapter requirement | Keep framework-neutral. The concrete process API belongs to each adapter. |
| Tool names, prompt hooks, approval dialogs, renderers, startup events, and framework todo projection | Adapter implementation detail | Keep only in the adapter repository or in a labeled implementation handoff. |
| Exact command and flag mappings | Adapter implementation detail | Generate or maintain from the supported CLI surface. Do not present one framework's schema as universal guidance. |
| This repository's Bun, release, tags, Sideshow, delegation, preview-slot, and exact commit-cadence rules | Repository-specific policy | Keep the repository-specific policy in active `.tandem` rules. Generic guidance can explain why durable `.tandem` data needs coherent, regular commits without imposing this workspace's cadence on every adapter. |
| Repeated hierarchy, decision, and lifecycle prose copied into several adapter prompt locations | Duplicate/obvious | Replace with concise operational statements and links or generated shared text. Test behavior, not incidental prose. |
| Treat persisted `accord.status: ready` as a current command action | Obsolete | Preserve legacy reads only. New work claims from the missing/unclaimed state through Tandem's supported lifecycle operation. |
| Parse frontmatter, construct IDs, infer roles from ID shape, bypass the CLI, or override actor identity | Unsupported | Do not implement in an adapter. These violate the Tandem ownership boundary. |
| Assume command availability authorizes claim, acceptance, cancellation, completion, merge, push, or cleanup | Unsupported | Require authority from the current assignment, responsible caller, or applicable workspace policy. |

## Active Tandem rule category audit

This audit covers all 12 active rules read from the main Tandem workspace on 2026-08-03. It evaluates their stored categories against the normative meanings above. It does not change `.tandem` state. IDs are category-local; the parent/orchestrator should let Tandem allocate IDs for any new split rules.

| Current rule | Category audit | Proposed disposition |
| --- | --- | --- |
| `always:6` — use Bun by default unless incompatible | The text defines a default with a concrete deviation condition. That is `prefer`, not an invariant. | **Migrate to `prefer`.** Proposed text: “Prefer Bun as the package manager and script runner for JavaScript/TypeScript automation, including docs-site recipes and CI. Use another tool when a concrete incompatibility makes Bun impractical.” Preserve source `decision-2`. |
| `always:7` — release requires an annotated tag and GitHub Release; a tag alone is not complete | The first clause is required action. The second is an independently enforceable prohibition with an explicit tag-only exception. | **Split.** Keep an `always` rule: “When cutting a Tandem release, create and push both the annotated Git tag and the GitHub Release object for that tag.” Add a `never` rule: “Do not treat a pushed tag alone as a complete release unless the caller explicitly requests tag-only.” Preserve source `task-15` on both. |
| `never:1` — do not advance a newly created task without an explicit request; tests are evidence only | The second clause explains the authority boundary that supports the prohibition. It does not require an independent positive action. | **Keep in `never` without a split.** The category and combined text are consistent. |
| `never:2` — do not modify adapter implementation during core work; write generic guidance and handoffs | The adapter-code prohibition is `never`. The positive documentation and handoff requirement is an independently enforceable required action. | **Split.** Keep `never`: “Do not modify `extensions/pi-tandem/`, external Pi configuration, or any other agent/framework adapter implementation as part of core Tandem work unless an explicit adapter task authorizes it.” Add `always`: “During core Tandem work, specify needed cross-framework behavior in Tandem-owned protocol or guidance documents and create an explicit implementation handoff when an adapter change is needed.” |
| `prefer:1` — use one primary area tag first | This is a default taxonomy choice among valid tagging options. | **Keep in `prefer`.** Optionally change “Use” to “Prefer” for reader clarity; the stored category remains authoritative. |
| `prefer:2` — add only a few capability tags when useful | “A few” and “when they aid delegation” require judgment and define a default rather than a hard limit. | **Keep in `prefer`.** No split is needed. |
| `prefer:3` — make Sideshow mockups first wherever practical | “Wherever practical” explicitly permits a constraint-based deviation. | **Keep in `prefer`.** This remains repository-specific tooling policy. |
| `prefer:4` — orchestrator may auto-accept objective work; keep judgment-heavy work in validation | The first clause grants permission and is descriptive context. The second clause is a required authority boundary, not a preference. | **Split.** Add `context`: “For delegated non-visual, non-manual work with passing automated validation and no blockers, the orchestrator is authorized to accept and complete the task without additional human validation.” Add `never`: “Do not accept or complete delegated visual, UX, manual, high-risk, or ambiguous work without human review; keep it in validation.” Remove the current combined `prefer` rule. Preserve its source on both. |
| `prefer:5` — assess worktree need up front; use isolation for overlap; avoid shared trees | The up-front assessment is required for the stated orchestration condition. The checkout choice still uses judgment, and the shared-tree clause has explicit allowed cases. | **Split.** Add `always`: “Before orchestrating parallel or potentially overlapping delegated work, assess whether each worker needs an isolated branch or worktree.” Keep a shorter `prefer`: “Prefer separate worktrees for likely file overlap, visual/design experiments, release automation, or independently committed work; prefer a shared tree only for read-only or explicitly coordinated work.” Preserve the source on both. |
| `prefer:6` — commit durable Tandem state regularly; do not leave it uncommitted; never commit local runtime state | Commit timing and boundaries are judgment-based defaults. The final checkout-local data prohibition is absolute and independently enforceable. | **Split.** Keep the regular-commit and commit-boundary guidance in `prefer`. Add `never`: “Never commit checkout-local identity, caches, credentials, or other runtime state.” The parent can also phrase “do not leave important durable Tandem changes uncommitted for an extended period” as a separate `never` rule if strict enforcement is intended. Preserve the source. |
| `context:1` — configure and clear the delegated visual preview route | This is a conditional operating procedure with several commands. It is not descriptive context. | **Migrate to `always`.** Proposed text: “For delegated TUI or visual work, configure the repository's Git-local preview slot so the user runs only `just dev` from the normal checkout; route it to delegated code and a safe fixture, report no extra setup, and clear the route during cleanup.” Preserve source `task-132`. |
| `context:2` — use a specific workspace tab for Board validation; continue or escalate based on result | This is a conditional validation procedure and escalation directive. It is not descriptive context. | **Migrate to `always`.** Keep the current trigger and procedure. Optionally split a `context` fact that tab 2 is the designated Board-validation location, but the “use,” “run,” “inspect,” “continue,” and “escalate” clauses remain `always`. Preserve the source. |

### Proposed parent-applied rule changes

After review, the parent/orchestrator should apply these through Tandem rule operations, not by editing `.tandem/tandem.md`:

1. Migrate `always:6` to `prefer`.
2. Split `always:7` into one `always` requirement and one `never` prohibition.
3. Keep `never:1` unchanged.
4. Split `never:2` into the adapter prohibition under `never` and the generic-guidance/handoff requirement under `always`.
5. Keep `prefer:1`, `prefer:2`, and `prefer:3` in place.
6. Split `prefer:4` into a permission statement under `context` and a human-review prohibition under `never`.
7. Split `prefer:5` into an up-front assessment under `always` and checkout-choice guidance under `prefer`.
8. Split `prefer:6` so the absolute local-runtime-state prohibition moves to `never`; keep commit cadence repository-specific under `prefer`.
9. Migrate `context:1` and `context:2` to `always`, with optional descriptive facts retained separately as `context`.

These are proposed repository-policy migrations only. They do not change universal Tandem semantics, and their repository-specific wording must not be copied into generic adapter guidance. Generic commit hygiene remains appropriate when it does not impose this workspace's exact cadence.

## Handoff A: `pi-tandem` adapter

### Generic requirement

Consume the Tandem-owned agent and adapter contract without making Pi prompt text the authority for protocol semantics. Preserve active rule categories and expose enough read context for an agent to discover a workspace, inspect all rules, inspect task relationships, and consult Decisions and Logs before mutation.

### Rationale

The current adapter has useful command mappings, but its runtime and maintainer text repeats protocol semantics and repository policy. That duplication can drift. Pi-specific tools, prompt registration, rendering, and diagnostics remain adapter-owned; Tandem semantics do not.

### Acceptance behavior

- Runtime guidance directs agents to inspect workspace rules and explains the four categories with the same operational effect as the protocol.
- A prohibition is not weakened into `prefer` or `context` because of its wording or narrow trigger.
- Guidance distinguishes command capability, validation evidence, and actor authority.
- Workspace discovery does not initialize or upgrade implicitly.
- Read operations consume Tandem JSON and preserve CLI-returned IDs, roles, relationships, warnings, and rule categories.
- The adapter does not parse or mutate Tandem Markdown, manage actor identity, or reproduce repository-specific commit-frequency policy as universal guidance. It can link to Tandem's generic commit-hygiene guidance.
- Shared runtime guidance has one adapter-owned source. Maintainer docs explain architecture and link to the Tandem contract instead of copying it verbatim.
- Focused tests assert critical authority and category behavior without snapshotting incidental prose.

### Adapter-owned files likely affected

These paths are implementation targets for a later explicit adapter task. This task does not modify them.

- `extensions/pi-tandem/index.ts`
  - consolidate generated runtime guidance;
  - add concise category semantics and authority boundaries;
  - keep Pi tool names and hook behavior local;
  - remove repository commit-frequency policy from generic runtime text.
- `extensions/pi-tandem/README.md`
  - describe adapter architecture and guidance ownership;
  - link to `docs/guides/agents-and-adapters.md` and the normative protocol section;
  - avoid duplicating runtime prompt prose.
- `extensions/pi-tandem/pi-tandem.md`
  - inventory remaining unique content, migrate only current adapter-owned material, then remove this separate unconsumed guidance source if the later task confirms no runtime consumer.
- `extensions/pi-tandem/plan/spec.md` and `extensions/pi-tandem/plan/todo.md`
  - record the generic-contract dependency and completion of consolidation.
- `extensions/pi-tandem/tests/pi-runtime-smoke.ts`, `extensions/pi-tandem/tests/relationship-smoke.ts`, and `extensions/pi-tandem/tests/smoke.ts`
  - test rule-category preservation, no implicit initialization, CLI-owned relationships, and lifecycle authority statements at stable behavior boundaries.
- Canonical external Pi configuration
  - promote only through a separate explicit task after repository-local validation; do not edit personal configuration from this repository task.

## Handoff B: any other agent or editor adapter

### Generic requirement

Implement a thin transport and presentation layer over an installed Tandem implementation. Preserve the protocol result and workspace policy while translating them into the host framework's tool, command, approval, and rendering model.

### Rationale

A common consumption contract lets different frameworks coordinate the same workspace without creating competing protocol implementations or framework-specific lifecycle semantics.

### Acceptance behavior

- The adapter starts from an explicit working directory and reports workspace and version diagnostics.
- Initialization and upgrades require explicit authority.
- A rule-list read returns every category and stable rule ID without reclassification.
- Read APIs expose enough task, relationship, Decision, Rule, and Log context for planning.
- Mutation APIs invoke Tandem through non-interpolated arguments or an equivalent safe interface.
- The adapter preserves warnings and does not treat a technically allowed transition as authorized.
- Framework prompts and UI explain only how to use that adapter; they link to Tandem-owned semantics.
- Tests prove pass-through behavior and authority boundaries without implementing a second protocol parser.

### Adapter-owned files likely affected

Use the target adapter's equivalents of:

- command runner and tool schemas;
- workspace diagnostics;
- runtime prompt or instruction assembly;
- output renderer;
- maintainer README;
- behavior-focused integration tests.

Do not change Tandem protocol semantics inside those files. If the generic contract is insufficient, propose a Tandem documentation or protocol change first.
