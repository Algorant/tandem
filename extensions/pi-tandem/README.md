# pi-tandem

`pi-tandem` is a lightweight Pi extension for Tandem. It exposes LLM-callable `tdm_*` tools and a `/tandem` command by shelling out to an installed `tdm` CLI with `execFile` argument arrays.

## Requirements

- `tdm` installed on `$PATH`, or `TANDEM_TDM_BIN`/`TDM_BIN` set to the binary path.
- A Tandem workspace (`.tandem/tandem.md`) for normal task/log/rule/decision operations.
- Pi extension runtime dependencies supplied by Pi (`@earendil-works/pi-coding-agent`, `@earendil-works/pi-ai`, `typebox`).

## Architecture

```text
LLM → Pi tool call → pi-tandem extension → tdm CLI → .tandem workspace
```

The extension does not parse or mutate Tandem Markdown/frontmatter directly. It only builds safe `tdm` argument arrays, runs the CLI, parses JSON output from read commands when available, formats results, and adds diagnostics.

## Tools

### `tdm_status`

Diagnose `tdm` availability and the nearest `.tandem/tandem.md` workspace.

### `tdm_task`

Maps to:

```text
tdm list [filters] --json
tdm show <id> --json
tdm add --title <title> [--parent <id>] [--blocker <id>] [--reference <id>] [--related-file <path>] [--subtask <title>] ...
tdm move <id> --state <state>
tdm complete <id> --summary <text> ...
```

Read actions default to JSON. Mutation actions keep the CLI's human-readable output.

Relationship parameters map directly to Tandem protocol fields: `parent` → `parentId`, `blockers` → strict dependency IDs, `references` → related Tandem document IDs, `relatedFiles` → project paths, and `subtasks` → lightweight checklist items. Create/inspect parent and blocker documents before using their IDs; unresolved parent/blocker references are errors, while unresolved related references are warnings.

### `tdm_accord`

Maps to:

```text
tdm accord ready|claim|deliver|accept|rework|block|fail <id> ...
```

Use this for work-agreement lifecycle transitions. Agents should not accept or complete work unless explicitly instructed by the user/orchestrator.

### `tdm_log`

Maps to:

```text
tdm log list --json
tdm log show <id> --json
tdm log search <query> --json
```

### `tdm_rules`

Maps to:

```text
tdm rules list [--category <category>] --json
tdm rules add --category <category> --rule <text> [--source <id>]
tdm rules edit --category <category> --id <id> --rule <text> [--source <id>]
tdm rules delete --category <category> --id <id>
```

### `tdm_decision`

Maps to:

```text
tdm decision list --json
tdm decision show <id> --json
tdm decision add --title <title> [--body <markdown>] [--reference <id>] [--tag <tag>]
```

### `tdm_search`

Maps to:

```text
tdm search <query> [--state <state>] [--type <type>] --json
```

## Slash command

```text
/tandem status
/tandem help
```

`/tandem status` checks `tdm --help`, workspace discovery, and `tdm list --json` when a workspace is present.

## Diagnostics

The extension reports common failure classes:

- missing `tdm` binary;
- missing `.tandem/tandem.md` workspace;
- unsupported/older CLI flags or subcommands;
- command timeout/abort;
- non-zero `tdm` command output with captured stdout/stderr.

## Local testing

From the repository root:

```text
cargo build --manifest-path tandem-tui/Cargo.toml
bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
bun extensions/pi-tandem/tests/smoke.ts
bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
bun extensions/pi-tandem/tests/relationship-smoke.ts
```

`smoke.ts` performs read-only checks against this repo's `.tandem` board, then mutating add/move/accord/rules/decision/log coverage in a temporary Tandem workspace. `pi-runtime-smoke.ts` temporarily creates an ignored project-local loader at `.pi/extensions/pi-tandem/index.ts`, starts `pi --mode rpc --approve --offline` with an isolated `PI_CODING_AGENT_DIR`, verifies fresh startup discovers `/tandem`, runs `/tandem status`, and removes the loader. `relationship-smoke.ts` creates a temporary parent/child/blocker/reference scenario through pi-tandem argument builders and `tdm`, then verifies persisted relationship fields and search visibility.

Manual project-local Pi smoke:

```text
mkdir -p .pi/extensions/pi-tandem
cat > .pi/extensions/pi-tandem/index.ts <<'EOF'
export { default } from "../../../extensions/pi-tandem/index";
EOF
TANDEM_TDM_BIN="$PWD/tandem-tui/target/debug/tdm" pi --approve
```

Then use `/tandem status` or `/reload` after edits. `.pi/` is ignored local runtime state; do not commit it unless a future task explicitly chooses a lightweight loader. For quick one-off testing without `.pi/`, use:

```text
TANDEM_TDM_BIN="$PWD/tandem-tui/target/debug/tdm" pi -e ./extensions/pi-tandem/index.ts
```

Do not promote this extension into `~/.pi/agent/extensions` until a separate review/promotion task.

## Prompt guidance

The extension registers prompt snippets/guidelines and appends focused guidance when a `.tandem/tandem.md` workspace exists or the prompt asks for durable coordination. See `pi-tandem.md` for the human-readable agent guidance.
