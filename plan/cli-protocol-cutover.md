# Tandem CLI and protocol cutover

Living design and cutover document for `task-246`.

This document records the interactive redesign of Tandem after sustained use.
Nothing in the current CLI, protocol, tests, adapters, or prior decisions is
preserved merely because it exists. Each behavior must earn its place.

## Working rules

- Prefer the smallest coherent product that handles normal Tandem work.
- No compatibility parser, migration fallback, backup format, dual path, or
  precautionary abstraction.
- A wrong decision is reverted or corrected when evidence appears.
- Protocol changes are allowed and may define a new protocol version.
- Decide the product model before designing the clap hierarchy.
- Record decisions provisionally until the whole model is coherent.
- Change normative `protocol/` semantics before Rust implementation.
- Core Tandem defines its behavior. Adapters follow it; adapter changes become
  Tasks in the owning workspace.
- `task-246` remains research and specification only. Implementation happens in
  follow-up Tasks.

## Decision ledger

| ID | Area | Status | Decision |
| --- | --- | --- | --- |
| D01 | Compatibility | provisional | The cutover may rewrite commands, flags, output, protocol fields, and prior v0 decisions. No fallback or compatibility path ships. |
| D02 | Parser | provisional | Remove the handwritten parser in one cutover. Do not ship two parsers. |
| D03 | Upgrade | provisional | Delete `tandem upgrade` and legacy protocol migration behavior. Unsupported old versions fail clearly. |
| D04 | Adapter authority | provisional | `pi-tandem` does not define core behavior. Reasonable missing capabilities are fixed in Tandem; adapter-only changes become Tasks in `~/.pi`. |
| D05 | Design order | provisional | Settle the protocol and command model interactively before selecting the clap type hierarchy. |
| D06 | Document types | provisional | Keep first-class `task` and `decision` documents. Decisions retain ADR metadata but use common document commands. |
| D07 | Papercuts | provisional | Remove Papercut as a protocol type. Represent friction as low-priority Tasks tagged `papercut`; retain a dedicated filtered TUI inbox and create a required `~/.pi` adapter/guidance handoff. |
| D08 | Task identity | provisional | Keep global `task-N` IDs for Epics and independently managed Tasks. Keep parent-derived `task-N-M` IDs for direct Subtasks because immediate human readability outweighs reparentability. Subtasks cannot be reparented; moving the work means canceling and recreating it. |
| D09 | Hierarchy roles | provisional | Keep exactly Epic → Task → Subtask. Epics are root-only; Tasks are root or direct Epic children; Subtasks are direct Task children and leaves. |
| D10 | Epic identity | provisional | Persist `kind: epic`; do not derive Epic identity from whether children currently exist. |
| D11 | Parent domain | provisional | Only task documents participate in strict `parentId` hierarchy. Decisions relate to work through `references`. |
| D12 | Lifecycle axes | provisional | Keep Task `state` and Accord `status` as separate concerns. Remove `review.status`; `state: validation` itself means human/product review is pending. |
| D13 | Active states | provisional | Keep exactly `todo`, `in-progress`, and `validation`. Blocking remains Accord metadata/status, not a Board column. |
| D14 | Review outcomes | provisional | Human acceptance from validation completes and archives immediately; requested changes return to `in-progress`; rejection cancels and archives. Reviewer, note, and timestamps remain metadata/events. |
| D15 | Validation purpose | provisional | Validation is an explicit, exceptional queue only when the orchestrator cannot verify a remaining acceptance criterion and needs human confirmation. Normal delivered work is verified, integrated, and completed directly. |
| D16 | Completion + Accord | provisional | Completing a Task with a delivered Accord atomically records Accord acceptance and archives the Task. There is no accepted-but-active state. |
| D17 | Validation entry | provisional | Escalation to validation requires the exact unresolved acceptance criterion and a note explaining why human judgment is required. |
| D18 | Validation outcomes | provisional | Human validation offers accept or request changes only. Accept completes; changes returns to `in-progress` with Accord `rework`. Ending work uses explicit cancel or fail; remove review reject. |
| D19 | Accord definition | provisional | Every Task has a mandatory Accord defining the agreed outcome, acceptance criteria, optional constraints/validation, and the agreement lifecycle. Accord is not delegation-only metadata. |
| D20 | Accord readiness | provisional | A Task cannot be ready without an explicit outcome and at least one acceptance criterion. Do not add a draft Accord status; incomplete ideas are not ready Tasks. |
| D21 | Release vs fail | provisional | Release returns viable work to Accord `ready` for reassignment. Fail means the agreed outcome cannot be achieved and atomically archives the Task as failed. |
| D22 | Accord definition fields | provisional | Task title/body carries outcome and context. Accord requires at least one acceptance criterion; constraints and planned validation are optional. |
| D23 | Assignee ownership | provisional | Keep only top-level `assignee`. Claim sets it; release clears it. Remove `accord.assignee`. |
| D24 | Accord transition text | provisional | Rework, block, release, and fail use one `accord.note`/`--note`; status and events identify the transition semantics. |
| D25 | Delivery proof | provisional | Accord delivery requires a summary and at least one evidence item. Evidence may be automated, observational, documentary, or reasoned. |
| D26 | Changed files | provisional | Delivery may include an optional structured `filesChanged` list; do not require it for non-code work. |
| D27 | Accord statuses | provisional | Active: `ready`, `claimed`, `delivered`, `rework`, `blocked`. Terminal in Logs: `accepted`, `failed`. Release returns to `ready`. |
| D28 | Add command | provisional | Use explicit `tandem add task` and `tandem add decision` typed subcommands. No default type. |
| D29 | Add title | provisional | The required title is the first quoted positional argument for both document types. |
| D30 | Body naming | provisional | Use `--body` for creation and update. Delete the `--description` one-off. |
| D31 | Unified show | provisional | One `show <id>` resolves Task or Decision documents from active or archived storage and renders by type. |
| D32 | Search command | provisional | Keep full-text `search` separate from structured `list`; their output and user intent differ. |
| D33 | Read scope | provisional | `list` and `search` share `--scope active|archived|all`, defaulting to active. Remove Log- and Decision-specific read commands. |
| D34 | Update command | provisional | Use one `update <id>`; resolve the document and validate fields by type. |
| D35 | Lifecycle separation | provisional | Generic update cannot write Task state or Accord status. Lifecycle actions own required notes, evidence, timestamps, assignment, and archival side effects. |
| D36 | List replacement | provisional | On update, an absent list field is unchanged, present repeated values replace the full list, and generic `--clear <field>` removes it. No add/remove flag matrix. |
| D37 | Accord command family | provisional | Keep `accord claim|deliver|rework|block|resume|release|fail` grouped under one major product concept. Remove Accord accept; complete owns acceptance. |
| D38 | Human review command | provisional | One `review <id>` action enters exceptional validation with criterion/note. `complete` accepts; `accord rework` requests changes. No review subcommand family. |
| D39 | Complete command | provisional | `complete` handles normal orchestrator-verified and exceptional human-validated acceptance based on current state; both accept the Accord and archive atomically. |
| D40 | Decision prose | provisional | Store ADR Context, Decision, Consequences, and Alternatives as Markdown body sections; remove dedicated prose flags. |
| D41 | Decision dates | provisional | Remove manual Decision date. Maintain automatic createdAt, updatedAt, and decidedAt when accepted/rejected. |
| D42 | Decision statuses | provisional | Keep proposed, accepted, rejected, deprecated, and superseded as Decision metadata editable through common update. |
| D43 | Rule categories | provisional | Keep always, never, prefer, and context as distinct agent enforcement vocabulary. |
| D44 | Rule identity | provisional | Keep composite category IDs such as always-12. Reclassification is delete-and-add because it changes meaning. |
| D45 | Rule provenance | provisional | Keep an optional source reference to the Task or Decision that established a Rule. |
| D46 | Storage layout | provisional | Store active Tasks in tasks/, durable Decisions in decisions/, one Rule per Markdown file in rules/, archived Tasks in logs/, and audit ledgers in events/. Board is a UI concept. |
| D47 | Archived resolution | provisional | Preserve the full Task and Accord, remove active state, and add only archivedAt plus resolution outcome/note/reviewer. Do not duplicate delivery data. |
| D48 | Decision durability | provisional | Decisions never move to Logs; rejected, deprecated, and superseded statuses preserve standing in decisions/. |
| D49 | Rule lookup | provisional | Common show resolves Rule IDs in addition to Tasks and Decisions; Rules retain their management family. |
| D50 | Event topology | provisional | Keep per-actor events/<actor-id>.jsonl only. Delete shared legacy events.jsonl support; retain ignored checkout-local actor identity. |
| D51 | Event coverage | provisional | Emit an event for every durable mutation and none for reads. |
| D52 | Event payload | provisional | Required envelope is ts, seq, actor, event, id plus event-specific structured data. Remove required free-text summary and never copy full bodies. |
| D53 | Global JSON | provisional | `--json` is a global option accepted before or after subcommands on every read and mutation. All commands share one envelope. |
| D54 | JSON streams | provisional | In JSON mode, success and error envelopes go to stdout and stderr remains empty; nonzero process status still signals failure. |
| D55 | Human streams | provisional | In human mode, primary results go to stdout; warnings and errors go to stderr. |
| D56 | Exit codes | provisional | Use 0 success, 2 grammar/usage error, and 1 for every operational failure. Stable JSON error codes provide finer categories. |
| D57 | Global aliases | provisional | Support only -h/--help, -V/--version, and -j/--json as short global aliases. No command aliases without observed demand. |
| D58 | Bare invocation | provisional | Running `tandem` with no args shows a concise landing/help surface; TUI remains explicit. |
| D59 | Prose values | provisional | Every human-written text field accepts leading hyphens as ordinary text; typed IDs/enums/numbers remain strict. |
| D60 | Scalar repetition | provisional | Repeating a scalar option is a usage error. Only declared list options may repeat. |
| D61 | Clearing fields | provisional | `--clear <field>` is the only clearing mechanism; supplied values must be nonempty. |
| D62 | TUI primary views | provisional | Top-level TUI views are Board, Rules, Decisions, and Logs. Do not add a speculative History/Activity view. |
| D63 | Papercut Board section | provisional | Preserve the current full-width Board list/subview model and add Papercuts as the fourth peer section beside Todo, In progress, and Validation. It is derived from tag=papercut; rows retain real state. |
| D64 | Validation visibility | provisional | Validation appears only as its existing Board subview section; no extra top-level tab or duplicate queue. |
| D65 | Web sequencing | provisional | Defer web redesign until the TUI information model is implemented and validated, then align web read models to it. |
| D66 | TUI structure | provisional | Preserve the current state-subview tabs, full-width selected-state rows, detail behavior, and State/Epic Board arrangement toggle. Do not replace it with kanban columns. |
| D67 | TUI cutover mutations | provisional | Remove TUI Add and direct Move. Keep and adapt existing Validation controls only; other Accord lifecycle actions remain CLI-only in this cutover. |
| D68 | Contextual lifecycle UI | proposed follow-up | Explore one contextual valid-actions picker in a separate TUI research Task; it is not part of the core cutover. |
| D69 | Protocol version | provisional | The redesigned protocol version is 0.3.0. |
| D70 | Historical Log schema | provisional | Active Tasks require Accord. Accord remains optional on canonical Logs so imported/pre-Accord history is not fabricated; new Logs naturally preserve Accord. |
| D71 | Existing workspaces | convention | Workspace owners handle existing 0.2.0 state project by project outside this cutover. No migration, archive, backup, conversion, ID-continuity, or recreation work belongs to this design or implementation plan. |
| D72 | Pi adapter boundary | handoff | Do not predetermine final Pi tool inventory. The ~/.pi overhaul Task audits and redesigns tools against the finished CLI; preserve convenient Papercut-tagged Task capture and keep all protocol logic in core. |
| D73 | Clap dependency | provisional | Adopt clap 4.6.x (current 4.6.6) with default features plus derive, using the normal compatible Cargo range. |
| D74 | Rust version policy | provisional | Do not add a rust-version package declaration as part of this cutover. |
| D75 | Generated artifacts | provisional | Defer shell completions and manpages. |
| D76 | JSON bootstrap | provisional | Extract exact -j/--json tokens before clap parsing so grammar failures honor JSON mode; literal reserved-token prose uses --flag=--json. |
| D77 | Validation ownership | provisional | Clap validates grammar and CLI-only enums; app/protocol retains all semantic vocabularies, transitions, hierarchy, and persistence validation. |
| D78 | Help under JSON | provisional | Help/version always emit generated human text and exit 0 even when --json is present. |
| D79 | Core implementation unit | provisional | Implement as one comprehensive Task on one isolated branch/worktree with ordered commits; review and merge only as a coherent candidate. |
| D80 | CLI module structure | provisional | Replace args.rs with ownership-based model.rs and parse.rs; retain commands.rs, output.rs, landing.rs, and a wiring-only mod.rs. Do not create one file per command. |
| D81 | Shared app errors | provisional | Replace app-layer CliError coupling with app::Error carrying stable operational kinds/details; CLI and TUI map it independently. |

## Interactive sequence

1. Document kinds, roles, identity, and storage.
2. Lifecycle model: workflow, accord, review, completion, cancellation.
3. Hierarchy: Epic, Task, Subtask, parentage, blockers, references.
4. Core command model: create, read, list/search, update, archive.
5. Rules and durable decisions.
6. Logs and events.
7. Output contract: human output, JSON, errors, exit codes, aliases, help.
8. TUI and web implications.
9. Protocol version and direct cutover behavior.
10. Cross-workspace handoffs.
11. Clap architecture, prototypes, tests, and implementation plan.

The sequence may revisit earlier provisional decisions when a later area exposes
a contradiction.

## 1. Document kinds, roles, identity, and storage

### Current model

| Record | Identity | Storage | Lifecycle | Hierarchy |
| --- | --- | --- | --- | --- |
| Task | `task-N` | `board/`, then `logs/` | workflow + accord + review + completion | yes |
| Epic | `task-N`, `type: task`, `kind: epic` | `board/`, then `logs/` | same as Task | root grouping role |
| Direct Epic Task | global `task-N` | `board/`, then `logs/` | same as Task | `parentId: <epic>` |
| Subtask | parent-derived `task-N-M` | `board/`, then `logs/` | same as Task | leaf below a Task |
| Decision | `decision-N`, `type: decision` | `board/` | ADR `status`, no task lifecycle | may be referenced or used as generic parent |
| Papercut | `papercut-N` | `papercuts/` | open/resolved | excluded |
| Completed/canceled work | original task ID | `logs/` | terminal completion record | cannot parent active work |

Current complexity:

- Role is derived from `type`, `kind`, resolved parent type, and parent role.
- ID shape does not establish role, but the derived role constrains ID shape.
- Epics and direct Epic children both use global `task-N` IDs.
- Only children directly beneath Tasks use parent-derived IDs.
- Reparenting is rejected if the immutable ID would no longer match the derived
  role.
- Papercuts are Markdown records with IDs, metadata, events, references, search,
  list/show/add/resolve operations, and their own directory — while the protocol
  says they are not documents.
- Decisions live on the Board but do not participate in Board workflow.
- Completed tasks remain addressable by their original IDs, and general `show`
  already resolves them from Logs.

### Design tests

A revised model should make these answers obvious:

- What records can users create?
- Does every record share one identity and lookup model?
- Which records participate in lifecycle and hierarchy?
- Does ID shape encode useful information or persistence structure?
- Can a record change role without changing identity?
- Does storage location describe record type, lifecycle, or both?
- Can `show <id>`, `list`, `add`, and `update` operate uniformly?

### Decisions

- **D06 — two first-class document types.** `task` and `decision` remain.
  Decision keeps ADR metadata and semantic `decision-N` identity, but its custom
  CLI family is removed in favor of common document commands.
- **D07 — Papercut becomes a Task convention.** A Papercut is a low-priority
  Task tagged `papercut`, not a protocol type. The TUI retains a dedicated
  Papercut inbox implemented as a saved active-Task filter. Quick capture
  creates the tagged Task; resolve uses normal complete/cancel behavior.
- **D08 — readable hierarchical Subtask identity.** Epics and independently
  managed Tasks keep global `task-N` IDs. A direct Subtask beneath `task-N`
  keeps `task-N-M`. A Subtask cannot be reparented because doing so would make
  its immutable ID lie. Moving the work means canceling it and creating a new
  Subtask under the new Task.

### Consequences

- Remove `.tandem/papercuts/`, `protocol::papercut`, Papercut-specific events,
  four Papercut commands, and its special search/JSON/output paths.
- Existing Papercut content needs a one-time direct conversion into Tasks in
  the new protocol version. This is the cutover itself, not a compatibility
  reader or fallback.
- `~/.pi` needs a handoff Task to replace `tandem_papercut` adapter behavior and
  update guidance about recording small friction.
- `show`, `list`, `add`, and `update` operate over Tasks and Decisions; the TUI
  may expose purpose-built filtered views without creating protocol types.
- Keep the current task allocator distinction: global `task-N` for Epics/Tasks,
  parent-derived `task-N-M` for Subtasks.
- Reparenting remains valid only when it does not change a record between Task
  and Subtask roles or invalidate its canonical ID.

### Rejected alternatives

- A first-class Papercut type: duplicates document, lifecycle, event, search,
  and command machinery for low-priority work.
- Removing Papercut capture entirely: loses a useful inbox workflow.
- Global IDs for every Task: structurally simpler, but loses immediate Subtask
  ownership in references and conversation.
- Stable global ID plus computed hierarchical alias: creates two identifiers,
  alias resolution, collision rules, and stale-display problems.
- Renumbering Subtasks on reparent: silently invalidates external references.

### Open questions

- Whether Epic, Task, and Subtask remain explicit/derived roles.
- Whether hierarchy remains exactly Epic → Task → Subtask or changes depth.
- Whether `kind: epic` remains persisted or is derived from structure.

## 2. Lifecycle model

### Current model

Every active Task can carry three parallel status axes:

- `state`: `todo`, `in-progress`, `validation`;
- `accord.status`: claimed, delivered, accepted, rework, failed, blocked;
- `review.status`: not-ready, pending, accepted, changes-requested, rejected.

Hand-written synchronization updates one field when another changes, and
`state_divergence_warning` exists because the fields can contradict each other.
Completion and cancellation then archive the Task into Logs with a fourth
terminal outcome concept.

### Decisions

- **D12 — two lifecycle axes.** Keep Task `state` and Accord `status` because
  they answer different questions. Task state is product/workflow position;
  Accord status is the worker/orchestrator agreement lifecycle. Remove
  `review.status`, which duplicates whether a Task is in validation.
- **D13 — three active Board states.** `todo`, `in-progress`, `validation`.
  Blocking is represented in Accord rather than as a fourth Board column.
- **D14 — review is transition plus evidence, not stored status.** Entering
  validation means human/product review is pending. Acceptance completes and
  archives immediately. Requested changes return to `in-progress`. Rejection
  cancels and archives. Reviewer, note, requested/decided timestamps, and event
  history survive without `review.status`.
- **D15 — validation is exceptional.** Validation exists only when the
  orchestrator cannot verify a remaining acceptance criterion and explicitly
  requires human confirmation. It is not the default destination for delivered
  work. Normal delivery is verified and integrated by the orchestrator, then
  completed directly into Logs once the Accord is met.

### Correction to the initial audit

The audit initially characterized all three axes as redundant and cited 90
nominal combinations. That was too broad. Some combinations are deliberate:
an orchestrator may accept a worker's Accord while product review remains
pending. The concrete inconsistency is narrower, such as `state: todo` with
`accord.status: claimed`, which current code detects through
`state_divergence_warning`. D12 removes the axis that actually duplicates
validation while preserving the distinct agreement lifecycle.

### Consequences

- Delete `review::STATUSES`, `review.status`, `reviewStatus`, and validation
  rules requiring `review.status: pending`.
- Validation becomes self-describing; no second field is required to prove it.
- Review request becomes an explicit escalation to validation plus the exact
  unresolved criterion, optional reviewer, and note metadata.
- Review acceptance calls normal completion/archive behavior rather than
  leaving an accepted Task active.
- The normal path bypasses validation: delivered Accord → orchestrator
  verification/integration → accepted Accord + completed Log.
- Human review should remain rare as agent verification improves.
- **D16 — complete accepts the Accord.** After orchestrator verification and
  integration, completing a delivered Task atomically writes Accord acceptance
  and moves the Task to Logs. There is no separate normal-path acceptance step
  and no accepted-but-active limbo.
- **D17 — validation requires a precise escalation.** Entering validation
  requires the exact unresolved acceptance criterion and a note explaining why
  the orchestrator cannot verify it.
- **D18 — two validation outcomes.** Human validation offers accept or request
  changes. Acceptance completes. Changes returns the Task to `in-progress` and
  sets Accord to rework with the note. Remove review reject; ending work is an
  explicit cancel or fail choice with a reason.

### Normal and exceptional flows

```text
normal:
  todo → claimed/in-progress → delivered → verify → integrate
       → complete (Accord accepted atomically) → Logs

human exception:
  delivered → validation(unresolved criterion + note)
            → accept → complete/Logs
            → changes → rework/in-progress

termination:
  cancel(reason) → Logs with canceled outcome
  fail(reason)   → Logs with failed outcome
```

### Rejected alternatives

- `review.status`: duplicates whether the Task is in validation and can drift.
- A default validation stage: turns human review into routine ceremony and
  scales poorly as agent verification improves.
- Separate Accord accept then Task complete: creates a two-step normal path and
  accepted-but-active limbo.
- Human review reject: does not distinguish continuing via rework from ending
  via cancel or fail. Current behavior is internally contradictory because it
  stores rejected while returning the Task to `in-progress`.
- Review changes writes `state: in-progress`; review rejection uses normal
  cancellation/archive behavior.
- Accord remains a distinct agreement lifecycle and is mandatory on every Task.

### Accord definition

- **D19 — mandatory definition of done.** Every Task has an Accord. The Task
  describes work and context; the Accord defines the agreed outcome,
  acceptance criteria, optional constraints and validation, then records claim,
  delivery, rework, and acceptance.
- **D20 — explicit readiness.** Accord `ready` is canonical, not legacy. A Task
  cannot become ready without an outcome and at least one acceptance criterion.
  There is no draft Accord status; incomplete ideas should not enter ready work.
- **D21 — release is not failure.** Release means the current agent cannot
  continue but the work remains viable; clear assignment and return the Accord
  to `ready`. Fail means the agreed outcome cannot be achieved; require a reason
  and atomically archive with Accord `failed` and completion outcome `failed`.

### Accord lifecycle

```text
ready → claimed → delivered → accepted/Logs
           ↕          ↓
        blocked     rework → claimed
           ↓
        release → ready

any active status → fail(reason) → failed/Logs
```

Active statuses are `ready`, `claimed`, `delivered`, `rework`, and `blocked`.
Terminal statuses `accepted` and `failed` occur only in Logs.

### Accord definition schema

- **D22 — acceptance is the only required definition field.** Task title/body
  carries outcome and context. Accord requires at least one acceptance
  criterion. `constraints` and planned `validation` remain optional. Do not
  require a separate outcome that repeats the Task prose.
- **D23 — one ownership field.** Keep top-level `assignee`; remove
  `accord.assignee`. Claim sets `assignee`, release clears it, and list/filter
  reads it directly. This eliminates two owners that can disagree.
- **D24 — one transition note.** Rework, block, release, and fail all write one
  current `accord.note` and accept one `--note` flag. The status and event name
  supply action semantics; the event ledger retains previous notes.

```yaml
accord:
  status: ready
  acceptance:                 # one or more, required
    - Every command supports --help without a workspace
  constraints:                # optional
    - No dual parser
  validation:                 # optional planned proof
    - cargo test --test cli_behavior
  note: ...                   # current exceptional transition explanation
  delivery:
    summary: ...              # required on delivery
    evidence:                 # one or more, required on delivery
      - cargo test --test cli_behavior: passed
    filesChanged: [...]       # optional
    deliveredAt: ...          # automatic
```

- **D25 — delivery must be verifiable.** Deliver requires a summary and at
  least one evidence item. Evidence may be an automated command result,
  rendered observation, document path, or reasoned verification.
- **D26 — changed files remain structured but optional.** Keep
  `delivery.filesChanged` for code audit and release notes; omit it naturally
  for non-code work.
- **D27 — final Accord status set.** Active: `ready`, `claimed`, `delivered`,
  `rework`, `blocked`. Terminal in Logs: `accepted`, `failed`. Release returns
  to `ready`.

## 3. Hierarchy

### Decisions

- **D09 — fixed three-level work hierarchy.** Epic → Task → Subtask. An Epic is
  root-only. A Task may be root-level or a direct Epic child. A Subtask is a
  direct Task child and a leaf. No arbitrary-depth task nesting.
- **D10 — explicit Epic marker.** Persist `kind: epic`. Epic identity does not
  appear or disappear based on whether children currently exist.
- **D11 — task-only parent hierarchy.** Only task documents may be targets or
  sources of `parentId`. Decisions connect to implementing work through
  `references`, never generic parentage.

### Consequences

- Delete generic parent relationships and validation branches.
- Delete support for a Task parented by a Decision or custom document.
- Keep `epic-task` and `subtask` as the only parent relationship classes.
- Keep global IDs for root Tasks and direct Epic Tasks; keep parent-derived IDs
  for Subtasks.
- Reject Epic parentage, children beneath Subtasks, arbitrary depth, and
  Subtask reparenting with direct structural errors.
- The TUI can render one bounded three-level tree without recursive role logic.

### Rejected alternatives

- Deriving Epic from children makes empty Epics impossible and changes role as
  children are added or completed.
- Task/Subtask-only hierarchy makes grouping Tasks indistinguishable from
  independently executable Tasks.
- Arbitrary-depth nesting complicates identity, delegation, completion,
  rendering, and child validation without evidence that deeper plans help.
- Decision parents conflate implementing a choice with hierarchy; references
  already express that relationship.

## 4. Core command model

### Creation

- **D28 — typed add subcommands.** Use `tandem add task` and
  `tandem add decision`. Explicit subcommands give each type accurate help and
  validation without conditionally-valid flags. There is no default type.
- **D29 — positional title.** The required title is the first quoted positional
  argument for both document types.
- **D30 — one Markdown body name.** Use `--body` on creation and update. Delete
  add's `--description` synonym.

```text
tandem add task "Fix command help" \
  --acceptance "Every command supports --help"

tandem add decision "Adopt clap" --body "..."
```

Do not add a `create` alias without observed demand.

### Read and browse

- **D31 — one document lookup.** `show <id>` resolves Tasks and Decisions from
  active or archived storage. Type controls rendering; location does not require
  a different command.
- **D32 — search remains distinct.** `list` performs structured browse/filter;
  `search` performs full-text discovery and returns match context.
- **D33 — common scope filter.** `list` and `search` accept
  `--scope active|archived|all`, default `active`. This replaces the `log` read
  family. `--type task|decision` replaces Decision-specific listing. Tagged
  Papercuts use ordinary `--tag papercut`.

```text
tandem show <id>
tandem list [filters] [--scope active|archived|all]
tandem search <query> [filters] [--scope active|archived|all]
```

### Metadata mutation

- **D34 — one inferred-type update.** `update <id>` resolves the document and
  validates supplied fields against Task or Decision semantics. The semantic ID
  already carries type, so a second type argument adds no information.
- **D35 — lifecycle is not metadata mutation.** `update` cannot write Task
  `state` or Accord `status`. Explicit lifecycle actions own required assignment,
  notes, delivery evidence, timestamps, events, and archival side effects.
- **D36 — deterministic list replacement (PC9).** An absent list field is
  unchanged. Present repeated flags define the exact resulting list. Generic
  `--clear <field>` removes a list or optional scalar. Do not create
  `--add-*`/`--remove-*` pairs for every field.

```text
# exact resulting tags are cli + config
tandem update task-12 --tag cli --tag config

# remove all references and clear the parent
tandem update task-12 --clear references --clear parent
```

### Lifecycle actions

- **D37 — Accord remains one command family.** Keep
  `accord claim|deliver|rework|block|resume|release|fail`. The family exposes one
  major product concept and its guarded state machine. Remove `accept` because
  D16 makes acceptance part of completion.
- **D38 — review is one escalation verb.** `review <id>` enters validation and
  requires unresolved criterion + note. It is not a status family. `complete`
  handles acceptance; `accord rework` handles requested changes.
- **D39 — one completion action.** `complete` handles both normal delivered work
  and validation. Normal completion records orchestrator verification;
  validation completion additionally requires human reviewer evidence. Both
  atomically accept Accord and archive.

```text
tandem accord claim <id> --assignee ...
tandem accord deliver <id> --summary ... --evidence ...
tandem accord rework <id> --note ...
tandem accord block <id> --note ...
tandem accord resume <id>
tandem accord release <id> --note ...
tandem accord fail <id> --note ...

tandem review <id> --criterion ... --note ... [--reviewer ...]
tandem complete <id> [--reviewer ...]
tandem cancel <id> --note ...
```

`resume` leaves blocked and returns to claimed. `release` clears assignee and
returns to ready. `fail` archives as failed. `cancel` archives work that is no
longer wanted or has been superseded.

## 5. Rules and decisions

### Decisions

- **D40 — ADR prose stays prose.** Context, Decision, Consequences, and
  Alternatives are Markdown body sections. Remove `--context`,
  `--consequence`, and `--alternative`; full-text search already covers them.
- **D41 — dates are automatic.** Remove manual `--date`. Maintain `createdAt`
  and `updatedAt`; write `decidedAt` when status becomes accepted or rejected.
- **D42 — retain standard ADR statuses.** `proposed`, `accepted`, `rejected`,
  `deprecated`, `superseded`. They are Decision metadata editable through
  common `update`, not Task workflow.
- Keep structured `deciders`, `supersedes`, `references`, and `tags` because
  they support attribution, relationships, and filtering.
- Remove stored `supersededBy`; store `supersedes` on the newer Decision and
  derive reverse relationships by lookup.

```yaml
id: decision-12
type: decision
title: Adopt clap
status: accepted
deciders: [ivan]
supersedes: [decision-3]
references: [task-246]
tags: [cli]
createdAt: ...
updatedAt: ...
decidedAt: ...
```

### Rules

- **D43 — retain four enforcement categories.** `always`, `never`, `prefer`,
  `context` remain distinct because they communicate agent behavior, not merely
  organization.
- **D44 — composite Rule identity.** Keep `always-12`-style IDs. Category is
  visible in references. Reclassification is delete-and-add because changing
  enforcement strength changes the Rule's meaning.
- **D45 — retain provenance.** Keep optional `source` pointing to the Task or
  Decision that established the Rule.
- Rules stay in workspace config, use hard deletion, and do not become
  documents or archived work.

```text
tandem rules list [always|never|prefer|context]
tandem rules add always "Read active rules before work" [--source task-12]
tandem rules edit always-12 "Revised rule text" [--source task-15]
tandem rules delete always-12
```

## 6. Logs and events

### Storage layout

- **D46 — persistence follows record purpose.** Use one Markdown file per
  durable record under type/purpose directories. Board remains a UI concept,
  not a filesystem directory.

```text
.tandem/
  tasks/       # active Tasks
  decisions/   # durable Decisions
  rules/       # always-12.md, prefer-7.md, ...
  logs/        # archived Tasks
  events/      # per-actor audit ledgers
  tandem.md    # workspace configuration
```

- Rule files carry `id`, `category`, optional `source`, timestamps, and the Rule
  text as Markdown body. This removes concurrent Rule mutation from the shared
  workspace config file.
- **D49 — common lookup includes Rules.** `show always-12` resolves a Rule.
  Rules retain their management family for category allocation and hard delete.

### Archived Tasks

- **D47 — minimal resolution metadata.** Move the complete Task and Accord to
  `logs/`, remove active `state`, and add only terminal facts. Accord delivery
  remains the sole source for summary, evidence, and changed files.
- **D48 — Decisions remain durable.** Decisions never move to Logs; rejected,
  deprecated, and superseded statuses communicate standing in `decisions/`.

```yaml
archivedAt: ...
resolution:
  outcome: completed | canceled | failed
  note: ...       # required for cancel/fail
  reviewer: ...   # human-validation completion only
```

### Event ledger

- **D50 — per-actor JSONL only.** Keep
  `.tandem/events/<actor-id>.jsonl` and ignored checkout-local
  `.tandem/actor-id`. Independent worktrees append separately. Delete legacy
  shared `.tandem/events.jsonl` support with no fallback reader.
- **D51 — every durable mutation emits.** Create, content/metadata update,
  lifecycle transition, archive, Decision change, and Rule change emit events.
  Reads never emit.
- **D52 — structured facts, not required prose.** Required envelope:
  `ts`, `seq`, `actor`, `event`, `id`, plus event-specific `data`. Remove
  required `summary`; UI/CLI renders a sentence from event type and data. Do not
  copy full Markdown bodies into events. Ordinary update data lists changed
  fields; lifecycle events retain transition values, notes, and terminal facts.

```json
{"ts":"...","seq":4,"actor":"a7b...","event":"task.updated","id":"task-12","data":{"fields":["tags","priority"]}}
{"ts":"...","seq":5,"actor":"a7b...","event":"accord.rework","id":"task-12","data":{"from":"delivered","to":"rework","note":"Help still requires a workspace"}}
```

## 7. Output contract

### JSON and process streams

- **D53 — JSON is global.** `--json` is accepted before or after subcommands on
  every read and mutation. Delete per-command JSON declarations and shapes.
- **D54 — one JSON stream.** Success and failure envelopes go to stdout; stderr
  stays empty. Process exit remains nonzero on failure.
- **D55 — clean human streams.** Human primary results go to stdout. Warnings
  and errors go to stderr, keeping piped result output clean.

```json
{"ok":true,"data":{},"warnings":[]}
{"ok":false,"error":{"code":"not_found","message":"record not found: missing","details":{"id":"missing"}}}
```

Every adapter requests JSON and never parses human output.

### Errors, help, aliases, and values

- **D56 — three process statuses.** `0` success, `2` grammar/usage failure,
  `1` every operational failure. Stable JSON error codes such as `usage`,
  `not_found`, `validation`, `conflict`, and `io` provide machine detail without
  multiplying exit statuses.
- **D57 — three universal short aliases.** `-h`/`--help`, `-V`/`--version`,
  `-j`/`--json`. Command flags and names stay canonical; no speculative
  `create`, `ls`, or other aliases.
- **D58 — safe bare invocation.** `tandem` prints the concise landing/help
  surface. It never conditionally launches TUI based on terminal detection.
- Every command and family exposes generated help without workspace discovery.
  Parser errors show relevant usage and a help hint; runtime errors do not dump
  unrelated usage.

- **D59 — prose values accept leading hyphens.** Titles, bodies, notes,
  acceptance criteria, summaries, evidence, and Rule text all follow one text
  policy. Typed IDs, enums, paths, and numbers remain strict.
- **D60 — duplicate scalars are errors.** `--priority high --priority low`
  fails as usage. Only declared list options such as `--tag` repeat.
- **D61 — explicit clearing only.** `--clear body`, `--clear tags`,
  `--clear parent`, etc. remove optional values. Supplied values must be
  nonempty; delete `--body ""` as a second clearing path.
- Support both `--flag value` and `--flag=value` through clap.
- Generated help includes one concise example where workflow is not obvious.

## 8. TUI and web implications

### Navigation

- **D62 — four primary views.** Board, Logs, Rules, Decisions. Keep the top-level
  name Logs because no global Activity view is justified; events appear in each
  record's detail timeline.
- **D63 — Papercuts is the fourth Board section.** Preserve the current state
  subview tabs and full-width list. Add Papercuts beside Todo, In progress, and
  Validation. It selects active Tasks tagged `papercut`; rows show their actual
  Task state and Accord status. Normal state sections exclude tagged Papercuts.
- **D64 — validation keeps its existing section only.** No duplicate top-level
  tab, global count surface, or separate queue.
- **D65 — web follows later.** Defer web redesign until the changed TUI is
  implemented and validated. Then align read models and taxonomy to the TUI;
  web remains read-only and is not a priority in the core cutover.

```text
[1] Board  [2] Logs  [3] Rules  [4] Decisions

[ Todo 5 ] [ In progress 2 ] [ Validation 1 ] [ Papercuts 4 ]

ID          PRI   ACCORD     RELATION   TITLE                 ASSIGNEE
...
```

### Board hierarchy and actions

- **D66 — preserve the current Board architecture.** State subview tabs,
  full-width selected-state rows, detail behavior, and the State Board / Epic
  Board arrangement toggle stay. The earlier kanban-column and Epic-focus mocks
  were an unsupported redesign and are rejected.
- **D67 — minimum mutation adaptation.** Remove `a Add` because TUI creation is
  not currently needed. Remove `m Move` because direct state movement no longer
  exists. Keep `v Validate`, adapted to request exceptional human review,
  complete on acceptance, or Accord rework on requested changes. Other Accord
  lifecycle operations remain CLI-only for this cutover.
- Keep filters, `$EDITOR` metadata/body editing, Logs/Rules/Decisions behavior,
  detail, themes, keyboard navigation, mouse hit maps, and current layout unless
  a required protocol field forces a targeted change.
- **D68 — contextual lifecycle picker is a follow-up proposal.** The rendered
  valid-actions picker is promising but not a priority for this cutover. Create
  a separate exploratory TUI research Task rather than expanding core scope.
- Do not add Task or Papercut quick capture in this cutover.

## 9. Protocol version and direct cutover

### Version and transition strategy

- **D69 — protocol 0.3.0.** This remains an explicitly pre-1.0 protocol while
  Tandem is actively reshapeable.
- **D70 — historical Logs need no fabricated Accord.** Active Tasks require
  Accord. Canonical Logs permit Accord to be absent because imported history
  may predate the agreement model. Every Log created by 0.3.0 naturally retains
  the Task's Accord.
- **D71 — existing workspace handling is outside this cutover.** The project
  owner handles old 0.2.0 state project by project. This document establishes
  the new convention only. It does not prescribe or implement migration,
  archive, backup, conversion, ID continuity, record recreation, tags, renamed
  directories, or temporary tooling.
- No `upgrade`, `migrate`, implicit converter, compatibility reader, backup
  directory convention, or dual protocol implementation ships. An encountered
  0.2.0 workspace fails clearly with detected and required protocol versions.

### Protocol 0.3.0 convention

A newly initialized workspace contains the D46 layout, D19–D27 Accord model,
D47 Log shape, and D50–D52 event ledger. Runtime behavior is defined only for
0.3.0. How any existing project chooses to retain or replace prior coordination
state is not a Tandem protocol concern.

## 10. Cross-workspace handoffs

### Pi adapter overhaul

Do not pre-design the final Pi tool inventory in this workspace. CLI command
consolidation does not imply that intent-shaped agent tools must disappear, and
preserving old tool names does not imply that old protocol behavior survives.
The owning implementation Task must make those choices against the finished
CLI.

Required handoff:

**Title:** Overhaul pi-tandem for Tandem protocol 0.3.0 and the comprehensive
CLI cutover

**Owning workspace:** `~/.pi`

**Implementation source:**
`~/.dotfiles/pi/.pi/agent/extensions/pi-tandem/`

**Blocker:** completed and installed core protocol 0.3.0/CLI cutover

**Goal:** Audit and refactor the entire Pi integration against the installed
canonical CLI. Map agent intents to the new command and JSON contracts. Retain,
combine, create, or remove tools based on model-call utility rather than
one-to-one CLI mirroring.

**Required scope:**

- inspect every current tool schema, argument builder, renderer, command,
  manifest entry, alias, test, README section, prompt guideline, and skill rule;
- use `execFile`/argument arrays and global `--json` envelopes exclusively;
- never parse or mutate Tandem Markdown, storage directories, events, IDs,
  hierarchy, Accord, lifecycle, or Rule files in TypeScript;
- remove old assumptions about `move`, per-command JSON, Review status,
  `accord accept`, separate Log/Decision/Papercut CLI families, additive list
  updates, `--description`, and current Rule argument shape;
- preserve a convenient agent intent for adding Papercut-tagged low-priority
  Tasks with mandatory Accord acceptance, whether through a retained
  `tandem_papercut` tool or a better schema chosen in that Task;
- evaluate intent-shaped `tandem_log`, `tandem_decision`, `tandem_task`, and
  other tools on their merits at implementation time; this document does not
  predetermine their survival or exact names;
- update `pi-tandem` skill guidance, `pi-agency` validation guidance,
  `pi-cc-use` aliases, config manifest, tests, README, widget/help inventory,
  and any generated/projection surfaces;
- validate real tool calls against an installed protocol 0.3.0 CLI, including
  creation, Accord delivery, exceptional human review, archived scope, Rule
  operations, Decision authoring, explicit clearing, errors, and Papercut
  capture.

**Acceptance:**

- no emitted argv references a removed command or flag;
- every tool parses the canonical JSON success/error envelope and respects
  process exit status;
- no integration code owns protocol or persistence behavior;
- Papercut capture remains convenient for agents;
- stale tools/guidance are removed rather than shimmed;
- retained and new tools have focused closed schemas and passing real-command
  tests.

**References:** `task-246`, the accepted protocol/CLI Decision, and the core
cutover Task ID once created.

**Related files:**

- `~/.dotfiles/pi/.pi/agent/extensions/pi-tandem/`
- `~/.dotfiles/pi/.pi/agent/skills/pi-tandem/SKILL.md`
- `~/.dotfiles/pi/.pi/agent/extensions/pi-agency/`
- `~/.dotfiles/pi/.pi/agent/extensions/pi-cc-use/`
- `~/.dotfiles/pi/.pi/agent/config-manifest.json`

Do not create this Task until the owner approves the complete handoff map. Do
not mutate the external workspace during `task-246`.

## 11. Clap architecture and implementation plan

### Dependency choice

- **D73 — clap 4.6.x.** Current stable is 4.6.6 (2026-08-06), MSRV 1.85,
  license MIT OR Apache-2.0. Use normal Cargo compatibility rather than exact
  pinning:

```toml
clap = { version = "4.6", features = ["derive"] }
```

Default features supply help, usage, color, error context, and suggestions.
Do not enable env, unicode, wrap_help, cargo, unstable, completion, or manpage
features.
- **D74 — no package MSRV declaration.** Clap has an effective dependency floor
  of Rust 1.85, but this cutover does not add `rust-version` or pin the user
  toolchain.
- **D75 — no generated shell/man artifacts.** Defer them until distribution has
  an observed need.

Measured disposable prototype:

- derive supports the full nested static tree and global `--json` before/after
  nested subcommands;
- prose values with leading hyphens parse; duplicate scalar options fail;
  repeatable lists append; empty values fail through a reusable value parser;
- root/family/leaf help and typo suggestions generate correctly;
- cold release build: about 5.1 seconds on the current machine;
- stripped minimal binary: 342 KB plain Rust versus 843 KB with clap, roughly
  501 KB isolated parser cost; current stripped Tandem is 4.63 MB;
- four clap-family packages enter the lockfile.

### Parser ownership and startup

- Use derive for the complete static command model. Do not maintain a builder
  command model in parallel. Narrow `Cli::command()` access is allowed for help
  tests and styles.
- Use `Cli::try_parse_from`; never let clap print or terminate the process.
- **D76 — bootstrap JSON mode.** Extract exact `-j`/`--json` argv tokens before
  parsing so grammar errors can use the JSON envelope. These tokens are globally
  reserved. Literal prose exactly equal to `--json` uses `--body=--json` (or the
  corresponding prose flag); ordinary leading-hyphen Markdown remains direct.
- **D77 — protocol semantics stay below CLI.** Clap validates grammar, required
  occurrence, repetition, numeric parsing, and CLI-owned `Scope`/`ClearField`.
  App/protocol validates priority, effort, Decision status, Accord transitions,
  Rule categories, hierarchy, references, versions, and filesystem state. Do
  not duplicate protocol enums as `ValueEnum`.
- **D78 — help/version remain text.** `--json --help` and `--json --version`
  print generated human documentation and exit 0; they are not result
  envelopes.
- Preserve `StartupRequest::{Exit,Tui,Web}`. Parsing and documentation exits
  never discover a workspace. TUI/Web open the project only after typed startup
  dispatch.

```text
raw argv
  → extract reserved -j/--json
  → Cli::try_parse_from
  → DisplayHelp/DisplayVersion → stdout, 0
  → grammar error → human/JSON renderer, 2
  → typed command → app operation → human/JSON renderer
  → StartupRequest for TUI/Web
```

### Implementation shape

- **D79 — one comprehensive core Task and branch.** Protocol, storage, app,
  clap CLI, tests, and required TUI adaptation overlap too heavily for separate
  partial merges. Use one isolated worktree/branch with ordered reviewable
  commits; review and merge only when coherent.
- **D80 — ownership-based CLI modules.** Delete `cli/args.rs`. Add `model.rs`
  for derive types and `parse.rs` for JSON bootstrap/clap mapping. Keep
  `commands.rs`, `output.rs`, `landing.rs`; shrink `mod.rs` to startup dispatch.
  Do not create one module per command.
- **D81 — shared operations return `app::Error`.** Remove shared app dependence
  on process-oriented `CliError`. `app::Error` carries stable operational kind,
  message, and optional details. CLI maps to exit/JSON/human output; TUI maps to
  UI feedback. Parse/usage errors remain CLI-owned.

```text
cli/
  model.rs
  parse.rs
  commands.rs
  output.rs
  landing.rs
  mod.rs
```

Implementation commit order:

1. Normative protocol 0.3.0 specification and Decision.
2. Rust protocol types/validation.
3. Project storage layout, discovery, events, and Rule files.
4. App operations, Accord lifecycle, archive behavior, and `app::Error`.
5. Clap dependency, derive model, parser/error bootstrap, typed dispatch.
6. Unified output/JSON, read scope, update replacement, help.
7. Process, protocol, app, and generated-help tests.
8. Targeted TUI adaptation preserving current structure.
9. Docs, landing, release notes, and removal of superseded code.

Rollback boundary is the isolated branch. If the candidate fails review, discard
or rework it. Never merge a dual parser, protocol compatibility path, or partial
cutover.

## Related evidence

- [`tandem/plan/clap-migration-research.md`](../tandem/plan/clap-migration-research.md)
- `task-246`
- `papercut-1`
- `papercut-9`
