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
tandem add --title <title> [--parent <id>] [--blocker <id>] [--reference <id>] [--related-file <path>] [--subtask <title>] ...
tandem move <id> --state <state>
tandem complete <id> --summary <text> ...
```

Read actions default to JSON. Mutation actions keep the CLI's human-readable output.

Relationship parameters map directly to Tandem protocol fields: `parent` → `parentId`, `blockers` → strict dependency IDs, `references` → related Tandem document IDs, `relatedFiles` → project paths, and `subtasks` → lightweight checklist items. Create/inspect parent and blocker documents before using their IDs; unresolved parent/blocker references are errors, while unresolved related references are warnings.

### `tandem_accord`

Maps to:

```text
tandem accord ready|claim|deliver|accept|rework|block|fail <id> ...
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
tandem decision add --title <title> [--body <markdown>] [--reference <id>] [--tag <tag>]
```

### `tandem_search`

Maps to:

```text
tandem search <query> [--state <state>] [--type <type>] --json
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

`smoke.ts` performs read-only checks against this repo's `.tandem` board, then mutating add/move/accord/rules/decision/log coverage in a temporary Tandem workspace. `pi-runtime-smoke.ts` temporarily creates an ignored project-local loader at `.pi/extensions/pi-tandem/index.ts`, starts `pi --mode rpc --approve --offline` with an isolated `PI_CODING_AGENT_DIR`, verifies fresh startup discovers `/tandem`, runs `/tandem status`, and removes the loader. `relationship-smoke.ts` creates a temporary parent/child/blocker/reference scenario through pi-tandem argument builders and `tandem`, then verifies persisted relationship fields and search visibility.

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

## Prompt guidance

The extension registers prompt snippets/guidelines and appends focused guidance when a `.tandem/tandem.md` workspace exists or the prompt asks for durable coordination. Guidance tells agents to use `validation` for delivered work awaiting acceptance, to tolerate legacy `state: review` reads, and to keep workflow state distinct from `review:` metadata and accord status. See `pi-tandem.md` for the human-readable agent guidance.
