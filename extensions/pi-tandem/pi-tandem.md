# pi-tandem Agent Guidance

Use `pi-tandem` when a project has `.tandem/tandem.md` or when the user asks for durable project coordination in Tandem.

## Prefer the tools

- Use `tdm_status` to check `tdm` and workspace health.
- Use `tdm_task` for task list/show/add/move/complete.
- Use `tdm_accord` for ready/claim/deliver/accept/rework/block/fail transitions.
- Use `tdm_log` and `tdm_search` for completed-work history and project search.
- Use `tdm_rules` for project rules.
- Use `tdm_decision` for first-class decision documents.

Avoid editing `.tandem/board/*.md`, `.tandem/logs/*.md`, or `.tandem/tandem.md` directly unless the user asks for raw source repair or the CLI cannot perform the needed action.

## Lifecycle cautions

- Claim or deliver only when assigned/asked.
- Do not accept accords, complete tasks, or archive work unless the user/orchestrator explicitly asks.
- Treat logs as first-class completed-work history, not as a trash folder.
- Keep review state, accord state, and task state distinct.

## Bootstrap behavior

If no workspace exists, ask before creating one. This MVP diagnoses missing workspaces but does not hide initialization policy inside the extension; use `tdm init --title <title>` only after user intent is clear.

## Adapter boundary

`pi-tandem` calls `tdm` through argument arrays. Tandem protocol behavior belongs in the Rust CLI/protocol docs, not in TypeScript adapter logic.
