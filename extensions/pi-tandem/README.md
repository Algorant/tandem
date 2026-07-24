# pi-tandem

`pi-tandem` is a lightweight Pi extension for Tandem. It exposes LLM-callable `tandem_*` tools and a `/tandem` command by shelling out to an installed `tandem` CLI with `execFile` argument arrays.

## Requirements

- `tandem` installed on `$PATH`, or `TANDEM_BIN` set to the binary path.
- A Tandem workspace (`.tandem/tandem.md`) for normal task/log/rule/decision operations.
- Pi extension runtime dependencies supplied by Pi (`@earendil-works/pi-coding-agent`, `@earendil-works/pi-ai`, `typebox`).

## Architecture

```text
LLM → Pi tool call → pi-tandem extension → tandem CLI → .tandem workspace
```

The extension does not parse or mutate Tandem Markdown/frontmatter directly. It only builds safe `tandem` argument arrays, runs the CLI, parses JSON output from read commands when available, formats results, and adds diagnostics.

## Tools

### `tandem_status`

Diagnose `tandem` availability and the nearest `.tandem/tandem.md` workspace.

### `tandem_task`

Maps to:

```text
tandem list [filters] --json
tandem show <id> --json
tandem add --title <title> [--kind epic] [--parent <id>] [--blocker <id>] [--reference <id>] [--related-file <path>] ...
tandem move <id> --state <state>
tandem update <id> [--title <title>] [--body <markdown>] [--kind epic] [--parent <id>] [--priority <priority>] [--tag <tag>] ...
tandem complete <id> --summary <text> ...
tandem cancel <id> --reason <text>
```

Read actions default to JSON. Mutation actions keep the CLI's human-readable output. For `action=update`, `body` maps to exact complete Markdown-body replacement without trimming; an explicit empty string clears the body. The add-only `description` convenience remains separate, and accord/review mutations continue through their dedicated lifecycle flows. `action=cancel` requires a reason and archives an auditable canceled Log; it is not permanent deletion and fails while active descendants remain.

Relationship parameters map directly to Tandem CLI fields: `kind` → `--kind`, `parent` → `--parent`/`parentId`, `blockers` → strict dependency IDs, `references` → related Tandem document IDs, and `relatedFiles` → project paths. Pi-tandem forwards these values and lets the CLI resolve the canonical role, allocate the ID, validate the graph, and return the relationship:

- a root `type: task`, `kind: epic` document is an Epic with a global `task-N` ID;
- a direct Epic child is a global-ID Task with `parentRelationship: "epic-task"`;
- a direct Task child is a leaf `<Task ID>-M` Subtask with `parentRelationship: "subtask"`;
- a decision/custom-parented task is a global-ID Task with generic `parentRelationship: "parent"`.

Allocation scans active tasks and completed logs so global and per-Task sequences are not reused. Subtasks cannot have children, Epics cannot have parents, direct Epic children never use hierarchical IDs, and role-changing or ID-invalidating reparenting is rejected. Create/inspect parent and blocker documents first; unresolved parent/blocker references are errors, while unresolved loose references are warnings.

Inline checklist `subtasks` metadata is legacy/deprecated and read-only through this adapter. `pi-tandem` does not expose it or forward `--subtask`; create lifecycle-bearing Subtask documents beneath their Task.

The adapter returns Tandem's read output without reclassifying relationships in TypeScript. List/search JSON retains CLI-computed `parentId`/`parentRelationship`; show returns `tasks` for an Epic, `subtasks` for a Task, and no child collection for a Subtask. There is no compatibility path for erroneous hierarchical IDs directly beneath Epics.

### `tandem_accord`

Maps to:

```text
tandem accord claim|deliver|accept|rework|block|fail <id> ...
```

Use this for work-agreement lifecycle transitions. Deliver finished agent work into the Validation workflow state (`state: validation`) for acceptance/rework decisions. Agents should not accept or complete work unless explicitly instructed by the user/orchestrator; automated validation evidence is not the same as human/product acceptance.

### `tandem_log`

Maps to:

```text
tandem log list --json
tandem log show <id> --json
tandem log search <query> --json
```

### `tandem_rules`

Maps to:

```text
tandem rules list [--category <category>] --json
tandem rules add --category <category> --rule <text> [--source <id>]
tandem rules edit --category <category> --id <id> --rule <text> [--source <id>]
tandem rules delete --category <category> --id <id>
```

### `tandem_decision`

Maps to:

```text
tandem decision list --json
tandem decision show <id> --json
tandem decision add --title <title> [--body <markdown>] [--status <status>] [--date <date>] [--decider <name>] [--context <text>] [--consequence <text>] [--alternative <text>] [--supersedes <decision-id>] [--superseded-by <decision-id>] [--reference <id>] [--tag <tag>]
```

Use this tool for ADR-compatible durable decisions. The document remains `type: decision`; ADR status and supersession belong in decision metadata/body sections, not task workflow `state` or a separate `adr` type.

### `tandem_search`

Maps to:

```text
tandem search <query> [--state <state>] [--type <type>] [--parent <id>] --json
```

## Slash command

```text
/tandem status
/tandem help
```

`/tandem status` checks `tandem --help`, workspace discovery, and `tandem list --json` when a workspace is present.

## Diagnostics

The extension reports common failure classes:

- missing `tandem` binary;
- missing `.tandem/tandem.md` workspace;
- unsupported/older CLI flags or subcommands;
- command timeout/abort;
- non-zero `tandem` command output with captured stdout/stderr.

## Local testing

From the repository root:

```text
cargo build --manifest-path tandem/Cargo.toml
bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
bun extensions/pi-tandem/tests/smoke.ts
bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
bun extensions/pi-tandem/tests/relationship-smoke.ts
```

`smoke.ts` performs read-only checks against this repo's `.tandem` board when the checkout has one, then mutating add/move/accord/rules/decision/log coverage in a temporary Tandem workspace. `pi-runtime-smoke.ts` temporarily creates an ignored project-local loader at `.pi/extensions/pi-tandem/index.ts` and, when needed, a disposable repository `.tandem` workspace containing an Epic, global Task, and parent-derived Subtask; it starts `pi --mode rpc --approve --offline`, verifies fresh startup discovers `/tandem`, runs `/tandem status`, and removes all temporary state. `relationship-smoke.ts` verifies generated Task-only/thin-adapter guidance, canonical Epic → Task → Subtask IDs, Board+Logs Task summaries, CLI-returned `epic-task`/`subtask`/generic `parent` output, completed-log sequence continuity, exact-parent reads, and rejection of nested Epics, children beneath Subtasks, role-changing reparenting, erroneous hierarchical Epic children, and erroneous global-ID Subtasks.

Manual project-local Pi smoke:

```text
mkdir -p .pi/extensions/pi-tandem
cat > .pi/extensions/pi-tandem/index.ts <<'EOF'
export { default } from "../../../extensions/pi-tandem/index";
EOF
TANDEM_BIN="$PWD/tandem/target/debug/tandem" pi --approve
```

Then use `/tandem status` or `/reload` after edits. `.pi/` is ignored local runtime state; do not commit it unless a future task explicitly chooses a lightweight loader. For quick one-off testing without `.pi/`, use:

```text
TANDEM_BIN="$PWD/tandem/target/debug/tandem" pi -e ./extensions/pi-tandem/index.ts
```

Do not promote this extension into `~/.pi/agent/extensions` until a separate review/promotion task.

## Prompt guidance and delegation

The extension registers prompt snippets/guidelines and appends focused guidance when a `.tandem/tandem.md` workspace exists or durable coordination is requested. Guidance describes the strict Epic → global Task → parent-derived leaf Subtask hierarchy and consumes Tandem's `epic-task`, `subtask`, and generic `parent` output without TypeScript reclassification.

Only Tasks are delegation roots initially. A delegated Task worker owns its direct Subtasks through the worker-session todo projection and produces one Task-root handoff; Epics and Subtasks are not independently delegated. Lifecycle authority stays with the parent/orchestrator.

See [`pi-tandem.md`](pi-tandem.md) for agent guidance and [`../../plan/delegated-task-tree-worker-spec.md`](../../plan/delegated-task-tree-worker-spec.md) for the explicit cross-repository Pi-config handoff. Keep that Pi-config implementation separate; do not modify personal dotfiles from this repository.
