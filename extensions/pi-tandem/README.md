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
tdm add --title <title> ...
tdm move <id> --state <state>
tdm complete <id> --summary <text> ...
```

Read actions default to JSON. Mutation actions keep the CLI's human-readable output.

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
bun --check extensions/pi-tandem/index.ts
bun extensions/pi-tandem/tests/smoke.ts
```

Optional local Pi runtime smoke:

```text
pi -e extensions/pi-tandem/index.ts
```

Then use `/tandem status` or ask Pi to list Tandem tasks. Do not promote this extension into `~/.pi/agent/extensions` until a separate review/promotion task.

## Prompt guidance

The extension registers prompt snippets/guidelines and appends focused guidance when a `.tandem/tandem.md` workspace exists or the prompt asks for durable coordination. See `pi-tandem.md` for the human-readable agent guidance.
