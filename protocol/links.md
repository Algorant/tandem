# Typed links

Protocol 0.5.0 adds typed links: an optional, Task-owned `links` field that names how one record relates to another. Links are read-model-visible, validated, and mutated through `tandem link` and `tandem complete --fixed-by`. A board without any `links` field is valid unchanged, so upgrading needs no migration step.

## Relationship to `references`

`references` is unchanged. It remains the untyped, loose list of related document IDs and absolute `http(s)` URLs, may name any record, warns only when an ID is unresolved, and is replaced wholesale by `update --reference`. Use a typed link when the relationship has a meaning Tandem should understand (a papercut that another Task fixed, a duplicate, a successor); use `references` for everything else, including URLs. A link is never inferred from a reference and a reference is never converted into a link. Hierarchy (`parentId`) and `blockers` are also separate and unchanged.

## Storage

Links are stored on the source record as a nested map keyed by link type; each value is a list of target document IDs.

```yaml
links:
  fixed-by: ["task-12"]
  relates-to: ["task-3", "decision-2"]
```

- Only a Task, Epic, or Subtask can own `links`. Decisions keep their own `supersedes` field, which is unrelated to the `supersedes` link type.
- Only the outgoing side is stored. The inverse is derived when reading and never written, so a link cannot disagree with its inverse.
- A target is the ID of any Task or Decision, active or archived. URLs are not link targets. Provisional IDs resolve to their current ID when a link is written, and sync rewrites a stale provisional ID like any other whole-token occurrence.
- An empty type list is not stored; removing the last link of a type removes its key, and removing the last link removes `links`.
- Sync merges each type's list as a set.

## Types

| Stored type | Derived inverse | Meaning of `A <type> B` |
| --- | --- | --- |
| `relates-to` | `relates-to` | A and B are related; symmetric. |
| `duplicates` | `duplicated-by` | A duplicates B. |
| `fixed-by` | `fixes` | B resolved A. |
| `fixes` | `fixed-by` | A resolves B. |
| `supersedes` | `superseded-by` | A replaces B. |

The vocabulary is fixed; any other type is invalid. `fixed-by` and `fixes` are both storable because either side can be the record being edited. Readers show both sides as one relation: if A stores `fixed-by: [B]`, B reads as `fixes A`.

## Validation

A link write is rejected, with no change, when:

- the type is not in the vocabulary;
- the target is empty;
- the source is not an active Task (a Decision, an archived record, or a missing ID);
- the target resolves to the source itself (a self-link, compared after provisional IDs are resolved);
- the target does not exist as a Task or Decision;
- the type is `fixed-by` and the target is archived with an outcome other than `completed`, because canceled or failed work fixes nothing. An archived `completed` target and any active target are valid for `fixed-by`; other types accept any existing target, archived or not.

Adding an existing link is a no-op reported as unchanged. Removing a link that is not stored is an error. Removal does not require the target to still resolve.

When reading, an unknown stored link type and an unresolved target of an active Board record produce warnings; archived Logs are immutable history and never warn, matching `references`.

## Reads

The record projection (`show --json`) includes `links` (outgoing, as stored) and `incomingLinks` (derived, labelled with the inverse type), each with the other record's title and location (`board` or `logs`). `list --link <type>` and `list --linked-to <id>` filter on the same combined view; `--link` accepts the stored types and the derived inverses.

## Resolving as fixed

`complete <id> --fixed-by <target>` records that the Task was resolved by another record. It validates exactly like `link add <id> fixed-by <target>`, then atomically adds the `fixed-by` link and archives the Task with the ordinary `resolution.outcome: completed`; there is no separate resolution outcome. Because the work was delivered by the linked record, the missing-delivery warning for the Task's own Accord is not emitted and its Accord is left as it was. The resolution note is `--note` when given, otherwise `Fixed by <target>`. The archived Task keeps the link as its durable citation.

## Events

`link add` and `link remove` append `task.linked` and `task.unlinked` with `data: {type, target}`. A changed link set is also an `updatedAt` change. `complete --fixed-by` appends the ordinary `task.completed` event only.
