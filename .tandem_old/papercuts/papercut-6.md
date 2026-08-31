---
id: papercut-6
title: "Auto-committer races the orchestrator and discards coordination commit messages"
status: open
createdAt: "2026-08-24T23:05:34Z"
updatedAt: "2026-08-24T23:05:34Z"
references: ["task-228", "task-240"]
tags: ["automation", "git", "tooling", "workflow"]
---
## Observed

After recording decision-11 and creating task-241, the orchestrator ran `git commit` with a written coordination message explaining the design session outcome. Git replied `nothing to commit, working tree clean`.

The auto-committer had already committed those `.tandem/` files as `chore(tandem): checkpoint metadata`. The authored message was never recorded. `just tidy-history` then correctly collapsed the adjacent generic commits into one generic commit.

The tool built in task-240 behaved correctly. The problem is upstream: the orchestrator cannot author a commit message for `.tandem/` changes, because the checkpointer commits them first.

## Effect

Every meaningful coordination change gets a generic message. Reconstructing why a decision or task was recorded requires reading the diff, since the commit log says only `checkpoint metadata`. This defeats much of the point of the tidy-up work.

Workaround used here: `git commit --amend -F -` on the unpushed commit to replace the message. That works only while the commit is unpushed and only if the orchestrator notices the race.

## Candidate remedies

- Let the orchestrator supply a message the checkpointer uses for its next commit.
- Have `tidy-history` prompt for or accept a message rather than always writing the generic one, so the collapse is also the moment the run gets named.
- Give the checkpointer a short debounce so a deliberate commit can win.

## Not a remedy

Disabling the checkpointer. Losing `.tandem/` state is worse than a generic message, which is already recorded in task-240's non-goals.
