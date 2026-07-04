---
title: Decisions and ADRs
description: Record durable Tandem decisions as ADR-compatible decision documents.
---

# Decisions and ADRs

Use Tandem `decision` documents for durable project, product, and architecture choices. A Tandem decision is ADR-compatible; do not create a separate `adr` type, task state, or completed-task log just to record an architecture decision.

## Frontmatter pattern

Required v0 fields are `id`, `type: decision`, and `title`. The CLI writes those fields plus timestamps, `references`, `tags`, and the Markdown body.

Optional ADR-friendly metadata may be preserved by tools and edited in Markdown when needed:

```yaml
id: decision-12
type: decision
title: Store completed work in logs
status: accepted # proposed | accepted | superseded | deprecated | rejected
date: 2026-07-01
deciders: [Algorant, Pi]
tags: [adr, protocol]
references:
  - task-87
  - decision-4
supersedes:
  - decision-4
supersededBy: []
createdAt: 2026-07-01T18:00:00Z
updatedAt: 2026-07-01T18:00:00Z
```

`status` is ADR record metadata, not workflow `state`. Use `references` for links the current CLI/TUI should find, including superseded or superseding decisions.

## Body template

```markdown
## Status

Accepted, proposed, superseded, deprecated, or rejected.

## Context

What forces, constraints, alternatives, or prior decisions made this choice necessary?

## Decision

What has been decided?

## Consequences

What becomes easier, harder, required, or intentionally deferred?

## Supersession

- Supersedes: decision-N or none
- Superseded by: decision-M or none

## References

- task-N, decision-N, log-N, code/docs paths, or external links as needed
```

## CLI example

```sh
body=$(cat <<'MD'
## Status

Accepted.

## Context

Tandem needs architecture decisions to be durable and searchable without adding another document type.

## Decision

Record ADR-compatible choices as Tandem `decision` documents.

## Consequences

Agents and humans use the same decision surface. Supersession links stay visible through `references`.

## Supersession

- Supersedes: none
- Superseded by: none
MD
)

tandem decision add \
  --title "Use Tandem decisions for ADRs" \
  --body "$body" \
  --reference task-87 \
  --tag adr

tandem decision list
tandem decision show decision-12 --json
```

## TUI workflow

1. Run `tandem tui`.
2. Press `4` or click **Decisions**.
3. Browse decision records in the list and read ADR sections in the body pane.
4. Press `a` to add a basic title/body decision. Include the body template above when recording an ADR-compatible decision.

The Decisions view should not invent decision lifecycle columns. Status and supersession are record metadata/body content, not Board workflow state.

## Pi agent pattern

Use the Pi tool, not raw `.tandem` edits, for normal creation:

```text
tandem_decision action=add title="Use Tandem decisions for ADRs" references=["task-87"] tags=["adr"] body="## Status\n\nAccepted.\n\n## Context\n..."
```

Use `tandem_search` or `tandem_decision action=show` to inspect existing decisions before creating a replacement. Do not model decisions as tasks, `state` values, accord statuses, or a separate `adr` type.
